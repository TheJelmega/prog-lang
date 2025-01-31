use crate::{ast::*, common::{NameId, NameTable, PrecedenceImportPath, Symbol}, error_warning::ErrorCode, literals::{Literal, LiteralTable}};

use super::{AstError, Context, ContextNodeData};




pub struct PrecedenceCollection<'a> {
    ctx:     &'a mut Context,
    names:   &'a NameTable,
}

impl<'a> PrecedenceCollection<'a> {
    pub fn new(ctx: &'a mut Context, names: &'a NameTable) -> Self {
        Self {
            ctx,
            names,
        }
    }
}

impl Visitor for PrecedenceCollection<'_> {
    fn visit_precedence(&mut self, ast: &Ast, node_id: AstNodeRef<Precedence>) where Self: Sized {
        let ctx_node = self.ctx.get_node_for(node_id);
        let scope = &ctx_node.scope;

        let node = &ast[node_id];
        let name = self.names[node.name].to_string();

        let id = {
            let mut precedences = self.ctx.precedences.write().unwrap();
            precedences.add_precedence(name.to_string())
        };

        {
            let mut syms = self.ctx.syms.write().unwrap();
            syms.add_precedence(&scope, name, id);
        }

        let ctx_node = self.ctx.get_node_for_mut(node_id);
        let ContextNodeData::Precedence(prec_id) = &mut ctx_node.data else { unreachable!() };
        *prec_id = id;
    }
}

pub struct PrecedenceAttribute<'a> {
    ctx:             &'a Context,
    lit_table:       &'a LiteralTable,
    builtin_name_id: NameId,
}

impl<'a> PrecedenceAttribute<'a> {
    pub fn new(ctx: &'a Context, names: &'a NameTable, literals: &'a LiteralTable) -> Self {
        let builtin_name_id = names.get_id_for_str("builtin");
        Self {
            ctx,
            lit_table: literals,
            builtin_name_id,
        }
    }
}

impl Visitor for PrecedenceAttribute<'_> {
    fn visit_precedence(&mut self, ast: &Ast, node_id: AstNodeRef<Precedence>) where Self: Sized {
        let node = &ast[node_id];

        let ContextNodeData::Precedence(id) = &self.ctx.get_node_for(node_id).data else { unreachable!() };

        for attr_id in &node.attrs {
            let attr = &ast[*attr_id];
            for meta in &attr.metas {
                match meta {
                    AttribMeta::Simple { .. } => self.ctx.add_error(AstError {
                            node_id: node_id.index(),
                            err: ErrorCode::AstInvalidAttribute { info: "Only the builtin attribute is allowed on precedences".to_string() },
                        }),
                    AttribMeta::Expr { .. } => self.ctx.add_error(AstError {
                        node_id: node_id.index(),
                        err: ErrorCode::AstInvalidAttribute { info: "Only the builtin attribute is allowed on precedences".to_string() },
                    }),
                    AttribMeta::Assign { path, expr } => {
                        let path = &ast[*path];
                        
                        if path.names.len() == 1 || path.names[0] == self.builtin_name_id {
                            let Expr::Literal(lit_node_id) = expr else { 
                                self.ctx.add_error(AstError {
                                    node_id: node_id.index(),
                                    err: ErrorCode::AstInvalidAttributeData { info: format!("Builtin attribute only accepts string literals") },
                                });
                                continue;
                            };

                            let LiteralValue::Lit(lit_id) = ast[*lit_node_id].literal else { 
                                self.ctx.add_error(AstError {
                                    node_id: node_id.index(),
                                    err: ErrorCode::AstInvalidAttributeData { info: format!("Builtin attribute only accepts string literals") },
                                });
                                continue;
                            };

                            let name = {
                                let lit = &self.lit_table[lit_id];
                                match lit {
                                    Literal::String(path) => path.to_string(),
                                    _ => {
                                        self.ctx.add_error(AstError {
                                            node_id: node_id.index(),
                                            err: ErrorCode::AstInvalidAttributeData { info: format!("Builtin attribute only accepts string literals") },
                                        });
                                        continue;
                                    },
                                }
                            };

                            match name.as_str() {
                                "lowest_precedence" => {
                                    let mut precedences = self.ctx.precedences.write().unwrap();
                                    precedences.set_lowest(*id);
                                },
                                "highest_precedence" => {
                                    let mut precedences = self.ctx.precedences.write().unwrap();
                                    precedences.set_highest(*id);
                                },
                                _ => {
                                    self.ctx.add_error(AstError {
                                        node_id: node_id.index(),
                                        err: ErrorCode::AstInvalidAttributeData { info: format!("Only 'highest_precedence' and 'lowest_precedence' are allowed builtin attributes on precedences") },
                                    });
                                    continue;
                                },
                            }
                            
                            
                        }
                    },
                    AttribMeta::Meta { .. } => self.ctx.add_error(AstError {
                        node_id: node_id.index(),
                        err: ErrorCode::AstInvalidAttribute { info: "Only the builtin attribute is allowed on precedences".to_string() },
                    }),
                }
            }
        }
    }
}

pub struct PrecedenceImportCollection<'a> {
    ctx:         &'a Context,
    names:       &'a NameTable,
    top_level:   bool,
    pub imports: Vec<PrecedenceImportPath>
}

impl<'a> PrecedenceImportCollection<'a> {
    pub fn new(ctx: &'a Context, names: &'a NameTable) -> Self {
        let top_level = ctx.mod_root.is_empty();
        Self {
            ctx,
            names,
            top_level,
            imports: Vec::new(),
        }
    }
}

impl Visitor for PrecedenceImportCollection<'_> {
    fn visit_module(&mut self, ast: &Ast, node_id: AstNodeRef<ModuleItem>) where Self: Sized {
        self.top_level = false;
        helpers::visit_module(self, ast, node_id);
    }

    fn visit_precedence_use(&mut self, ast: &Ast, node_id: AstNodeRef<PrecedenceUse>) where Self: Sized {
        if !self.top_level {
            let scope = &self.ctx.get_node_for(node_id).scope;

            let path = scope.to_string();

            self.ctx.add_error(AstError {
                node_id: node_id.index(),
                err: ErrorCode::AstNotTopLevel { 
                    path,
                    info: "Precedence use".to_string(),
                 }
            });
            return;
        }

        let node = &ast[node_id];
        
        let group = node.group.map(|group| self.names[group].to_string());
        let package = match node.package {
            Some(package) => self.names[package].to_string(),
            None          => self.ctx.lib_path.package.clone(),
        };
        let library = match node.library {
            Some(library) => self.names[library].to_string(),
            None => package.clone(),
        };

        for precedence in &node.precedences {
            let name = self.names[*precedence].to_string();

            let import_path = PrecedenceImportPath::new(group.clone(), package.clone(), library.clone(), name.clone());
            self.imports.push(import_path);
        }
    }
}

pub struct PrecedenceConnection<'a> {
    ctx:   &'a Context,
    names: &'a NameTable,
}

impl<'a> PrecedenceConnection<'a> {
    pub fn new(ctx: &'a Context, names: &'a NameTable) -> Self {
        Self {
            ctx,
            names,
        }
    }
}

impl Visitor for PrecedenceConnection<'_> {
    fn visit_precedence(&mut self, ast: &Ast, node_id: AstNodeRef<Precedence>) where Self: Sized {
        let node = &ast[node_id];
        let name = &self.names[node.name];

        let ctx_node = self.ctx.get_node_for(node_id);

        let syms = self.ctx.syms.read().unwrap();
        let sym = syms.get_symbol(&ctx_node.scope, name).unwrap();
        let Symbol::Precedence(sym) = &*sym.read() else {
            self.ctx.add_error(AstError {
                node_id: node_id.index(),
                err: ErrorCode::InternalError("Expected Precedence symbol when accessing symbol associated to a precedencenode ")
            });
            return;
        };

        let mut precedence_dag = self.ctx.precedences.write().unwrap();

        if let Some(lower) = &node.lower_than {
            let lower_id = precedence_dag.get_id(&self.names[*lower]);
            precedence_dag.set_order(lower_id, sym.id);
        }

        if let Some(higher) = &node.higher_than {
            let higher_id = precedence_dag.get_id(&self.names[*higher]);
            precedence_dag.set_order(sym.id, higher_id);
        }
    }
}