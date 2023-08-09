#[cfg(test)]
mod test;

use sqlx_core::HashMap;
use nom::branch::alt;
use nom::combinator::{consumed, map};
use nom::IResult;
use nom::bytes::complete::*;
use nom::character::complete::alphanumeric1;
use nom::multi::{many0, many1};
use nom::sequence::tuple;

/// View of miniKQL
#[derive(Debug, Clone)]
pub enum Node<'a> {
    /// Value with `'`. No idea about it. In that place, used for choose betwee list and struct
    Apos(Box<Node<'a>>),  
    /// Declaring of variable       
    Decl(&'a str, Box<Node<'a>>),
    /// reference to variable
    Ref(&'a str), 
    /// some scalar, may be string or number or quoted string             
    Scal(&'a str), 
     /// list of nested value             
    List(Vec<Node<'a>>),        
    /// named struct with nested values
    Struct(&'a str, Vec<Node<'a>>), 
}

impl<'a> Node<'a> {
    /// Parse string to Node tree
    pub fn parse(input: &'a str) -> IResult<&str, Node<'a>> {
        val(input)
    }
    fn eval_with_vars(&mut self, vars: &mut HashMap<&'a str, Self>) {
        match self {
            Node::Apos(v) => v.eval_with_vars(vars),
            Node::Decl(k, v) => {
                v.eval_with_vars(vars);
                let v = v.as_ref().clone();
                vars.insert(k,v);
                *self = Node::List(vec![]);
            },
            Node::Ref(k) => {
                if let Some(v) = vars.get(k) {
                    *self = v.clone();
                }
            },
            Node::List(list) | Node::Struct(_, list) => for v in list {
                v.eval_with_vars(vars);
            },
            Node::Scal(_) => {},
        }
    }
    /// Evaluate all references to declared values
    pub fn eval(&mut self) {
        let mut map = HashMap::new();
        self.eval_with_vars(&mut map);
    }
    /// Find node by text (struct or scalar)
    pub fn find(&self, key: &str) -> Option<&Self> {
        match self {
            Node::Apos(v) => v.find(key),
            v @ Node::Scal(name) if *name == key => Some(v),
            Node::Scal(name) => {
                //println!("scalar [{name}] not a {key}");
                None
            }
            v @ Node::Struct(name,_) if *name == key => Some(v),
            Node::List(list) | Node::Struct(_, list ) => {
                let mut candidate = None;
                for v in list {
                    if v.text() == Some(key) {
                        return Some(v);
                    }
                    if let Some(v) = v.find(key) {
                        candidate = Some(v);
                    }
                }
                candidate
            },
            _ => None,
        }
    }
    /// Invoke text of node (struct or scalar)
    pub fn text(&self) -> Option<&str> {
        match self {
            Node::Apos(v) => v.text(),
            Node::Scal(text) | Node::Struct(text, _) => Some(text),
            _ => None,
        }
    }
    /// invoke list of nested nodes
    pub fn list(&self) -> Option<&[Self]> {
        match self {
            Node::Apos(v) => v.list(),
            Node::List(list) | Node::Struct(_, list) => Some(list.as_slice()),
            _ => None,
        }
    }
}

pub fn invoke_outputs<'a, 'b: 'a>(node: &'b Node<'a>) -> Option<Vec<(usize, &'a str, &'a str, bool)>> {
    let outputs = node.find("return")?.find("KqpPhysicalQuery")?.find("KqpTxResultBinding")?.find("StructType")?.list()?;
    println!("{outputs:?}");
    let result = outputs.into_iter().enumerate().filter_map(|(n, item)| {
        let mut iter = item.list()?.into_iter();
        let mut is_optional = false;
        let name = iter.next()?.text()?;
        let typ = iter.next()?;
        let typ = if typ.text()? == "OptionalType" {
            is_optional = true;
            typ.list()?.into_iter().next()?
        } else {
            typ
        }.list()?.into_iter().next()?;
        Some((n, name, typ.text()?, is_optional))
    }).collect();
    Some(result)
}

fn extended_aplhanumeric1(i: &str) -> IResult<&str, &str> {
    let sym = map(consumed(many1(is_a("%_-="))), |(c,o)| c);
    let alpha_or_sym = alt((alphanumeric1, sym));
    let shuffle = many1(alpha_or_sym);
    map(consumed(shuffle), |(c,_)|c)(i)
}

fn spaces(i: &str) -> IResult<&str, ()> {
    let (i, _) = many0(is_a(" \t\r\n"))(i)?;
    Ok((i,()))
}

fn set_var<'a>(i: &'a str) -> IResult<&'a str, Node<'a>> {
    tuple(( tuple((spaces, tag("("), spaces)), tag("let "), spaces, tag("$"), alphanumeric1, tag(" "), val , spaces, tag(")")))(i)
        .map(|(i, (_, _, _, _, k, _, v, _, _))|(i, Node::Decl(k, Box::new(v))))
}

fn get_var<'a>(i: &'a str) -> IResult<&'a str, Node<'a>> {
    tuple(( spaces, tag("$"), alphanumeric1, spaces ))(i)
        .map(|(i,(_, _, name, _))|(i, Node::Ref(name)))
}

fn scalar_value(i: &str) -> IResult<&str, &str> {
    let braced = map(tuple((tag("\""),take_until("\""),tag("\""))), |( _, name, _)|name);
    let non_braced = |i| extended_aplhanumeric1(i);
    let result = tuple((
        alt((braced, non_braced)), spaces
    ));
    map(result, |(v, _)|v)(i)
}

fn scalar(i: &str) -> IResult<&str, Node<'_>> {
    scalar_value(i).map(|(i,name)|(i, Node::Scal(name)))
}

fn list(i: &str) -> IResult<&str, Node<'_>> {
    map(
        tuple(( tag("'("), many0(val), spaces, tag(")") )),
        |(_, list, _, _)| Node::Apos(Box::new(Node::List(list)))
    )(i)
}

fn strong_list(i: &str) -> IResult<&str, Node<'_>> {
    map(
        tuple(( tag("("), many0(val), spaces, tag(")") )),
        |(_,list,_, _)|Node::List(list)
    )(i)
}

fn stru(i: &str) -> IResult<&str, Node<'_>> {
    map(
        tuple(( tag("("), spaces, extended_aplhanumeric1, many1(val), tag(")") )),
        |(_,_,name, vals, _)|Node::Struct(name, vals)
    )(i)
}

fn apos(i: &str) -> IResult<&str, Node<'_>> {
    map(
        tuple(( tag("'"), val_no_spaces )),
        |(_, v)|Node::Apos(Box::new(v))
    )(i)
}

fn val_no_spaces(i: &str) -> IResult<&str, Node<'_>> {
    alt((
        set_var,
        get_var,
        list,
        stru,
        strong_list,
        scalar,
        apos,
    ))(i)
}

fn val(i: &str) -> IResult<&str, Node<'_>> {
    map(
        tuple(( spaces, val_no_spaces, spaces)),
        |(_, v, _)|v
    )(i)
}
