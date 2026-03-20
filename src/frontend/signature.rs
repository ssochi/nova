#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterDecl {
    pub names: Vec<String>,
    pub type_ref: TypeRef,
    pub variadic: bool,
}

impl ParameterDecl {
    pub(crate) fn render(&self) -> String {
        let prefix = if self.variadic { "..." } else { "" };
        format!(
            "{} {}{}",
            self.names.join(", "),
            prefix,
            self.type_ref.render()
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultDecl {
    pub names: Vec<String>,
    pub type_ref: TypeRef,
}

impl ResultDecl {
    fn render(&self) -> String {
        if self.names.is_empty() {
            self.type_ref.render()
        } else {
            format!("{} {}", self.names.join(", "), self.type_ref.render())
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeRef {
    Named(String),
    Interface,
    Slice(Box<TypeRef>),
    Chan(Box<TypeRef>),
    Map {
        key: Box<TypeRef>,
        value: Box<TypeRef>,
    },
}

impl TypeRef {
    pub fn render(&self) -> String {
        match self {
            TypeRef::Named(name) => name.clone(),
            TypeRef::Interface => "interface{}".to_string(),
            TypeRef::Slice(element) => format!("[]{}", element.render()),
            TypeRef::Chan(element) => format!("chan {}", element.render()),
            TypeRef::Map { key, value } => {
                format!("map[{}]{}", key.render(), value.render())
            }
        }
    }
}

pub(crate) fn render_result_decl_list(results: &[ResultDecl]) -> String {
    match results {
        [] => String::new(),
        [result] if result.names.is_empty() => format!(" {}", result.type_ref.render()),
        _ => format!(
            " ({})",
            results
                .iter()
                .map(ResultDecl::render)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}
