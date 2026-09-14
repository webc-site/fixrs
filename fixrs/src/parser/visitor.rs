use syn::{
  Attribute, Expr, FnArg, ForeignItem, ImplItem, Item, ItemMod, ItemUse, Macro, Pat, Path, Stmt,
  Token, TraitItem,
  parse::Parser,
  punctuated::Punctuated,
  visit::{
    Visit, visit_arm, visit_attribute, visit_block, visit_expr, visit_field, visit_foreign_item,
    visit_impl_item, visit_item, visit_item_mod, visit_item_use, visit_path, visit_stmt,
    visit_trait_item, visit_variant,
  },
};

use super::{
  collector::{ModuleContext, PathCollector, find_mod_insert_pos_and_indent},
  scope::child_scope,
};

impl<'ast> Visit<'ast> for PathCollector<'_> {
  fn visit_item(&mut self, i: &'ast Item) {
    let cfg = self.enter_cfg(item_attrs(i));
    let prev_len = self.local_idents.len();
    if let Item::Fn(f) = i {
      collect_sig_inputs(&f.sig, &mut self.local_idents);
    }
    visit_item(self, i);
    self.local_idents.truncate(prev_len);
    self.exit_cfg(cfg);
  }

  fn visit_impl_item(&mut self, i: &'ast syn::ImplItem) {
    let attrs = match i {
      ImplItem::Const(c) => &c.attrs[..],
      syn::ImplItem::Fn(f) => &f.attrs[..],
      syn::ImplItem::Type(t) => &t.attrs[..],
      syn::ImplItem::Macro(m) => &m.attrs[..],
      _ => &[],
    };
    let cfg = self.enter_cfg(attrs);
    let prev_len = self.local_idents.len();
    if let syn::ImplItem::Fn(f) = i {
      collect_sig_inputs(&f.sig, &mut self.local_idents);
    }
    visit_impl_item(self, i);
    self.local_idents.truncate(prev_len);
    self.exit_cfg(cfg);
  }

  fn visit_trait_item(&mut self, i: &'ast syn::TraitItem) {
    let attrs = match i {
      TraitItem::Const(c) => &c.attrs[..],
      syn::TraitItem::Fn(f) => &f.attrs[..],
      syn::TraitItem::Type(t) => &t.attrs[..],
      syn::TraitItem::Macro(m) => &m.attrs[..],
      _ => &[],
    };
    let cfg = self.enter_cfg(attrs);
    let prev_len = self.local_idents.len();
    if let syn::TraitItem::Fn(f) = i {
      collect_sig_inputs(&f.sig, &mut self.local_idents);
    }
    visit_trait_item(self, i);
    self.local_idents.truncate(prev_len);
    self.exit_cfg(cfg);
  }

  fn visit_expr(&mut self, i: &'ast syn::Expr) {
    let attrs = match i {
      Expr::Array(e) => &e.attrs,
      Expr::Assign(e) => &e.attrs,
      Expr::Async(e) => &e.attrs,
      Expr::Await(e) => &e.attrs,
      Expr::Binary(e) => &e.attrs,
      Expr::Block(e) => &e.attrs,
      Expr::Break(e) => &e.attrs,
      Expr::Call(e) => &e.attrs,
      Expr::Cast(e) => &e.attrs,
      Expr::Closure(e) => &e.attrs,
      Expr::Const(e) => &e.attrs,
      Expr::Continue(e) => &e.attrs,
      Expr::Field(e) => &e.attrs,
      Expr::ForLoop(e) => &e.attrs,
      Expr::Group(e) => &e.attrs,
      Expr::If(e) => &e.attrs,
      Expr::Index(e) => &e.attrs,
      Expr::Infer(e) => &e.attrs,
      Expr::Let(e) => &e.attrs,
      Expr::Lit(e) => &e.attrs,
      Expr::Loop(e) => &e.attrs,
      syn::Expr::Macro(e) => &e.attrs,
      Expr::Match(e) => &e.attrs,
      Expr::MethodCall(e) => &e.attrs,
      Expr::Paren(e) => &e.attrs,
      Expr::Path(e) => &e.attrs,
      Expr::Range(e) => &e.attrs,
      syn::Expr::Reference(e) => &e.attrs,
      Expr::Repeat(e) => &e.attrs,
      Expr::Return(e) => &e.attrs,
      syn::Expr::Struct(e) => &e.attrs,
      Expr::Try(e) => &e.attrs,
      Expr::TryBlock(e) => &e.attrs,
      syn::Expr::Tuple(e) => &e.attrs,
      Expr::Unary(e) => &e.attrs,
      Expr::Unsafe(e) => &e.attrs,
      Expr::While(e) => &e.attrs,
      Expr::Yield(e) => &e.attrs,
      _ => &[][..],
    };
    let cfg = self.enter_cfg(attrs);
    let prev_len = if let Expr::Closure(c) = i {
      let prev = self.local_idents.len();
      for input in &c.inputs {
        collect_pat_idents(input, &mut self.local_idents);
      }
      Some(prev)
    } else if let Expr::ForLoop(f) = i {
      let prev = self.local_idents.len();
      collect_pat_idents(&f.pat, &mut self.local_idents);
      Some(prev)
    } else if let Expr::Let(l) = i {
      let prev = self.local_idents.len();
      collect_pat_idents(&l.pat, &mut self.local_idents);
      Some(prev)
    } else {
      None
    };
    visit_expr(self, i);
    if let Some(prev) = prev_len {
      self.local_idents.truncate(prev);
    }
    self.exit_cfg(cfg);
  }

  fn visit_block(&mut self, i: &'ast syn::Block) {
    let prev_len = self.local_idents.len();
    visit_block(self, i);
    self.local_idents.truncate(prev_len);
  }

  fn visit_stmt(&mut self, i: &'ast syn::Stmt) {
    let attrs = match i {
      Stmt::Local(l) => &l.attrs[..],
      Stmt::Item(it) => item_attrs(it),
      Stmt::Expr(..) => &[],
      syn::Stmt::Macro(m) => &m.attrs[..],
    };
    let cfg = self.enter_cfg(attrs);
    if let Stmt::Local(l) = i {
      collect_pat_idents(&l.pat, &mut self.local_idents);
    } else if let Stmt::Item(it) = i {
      collect_item_idents(it, &mut self.local_idents);
    }
    visit_stmt(self, i);
    self.exit_cfg(cfg);
  }

  fn visit_arm(&mut self, i: &'ast syn::Arm) {
    let cfg = self.enter_cfg(&i.attrs);
    let prev_len = self.local_idents.len();
    collect_pat_idents(&i.pat, &mut self.local_idents);
    visit_arm(self, i);
    self.local_idents.truncate(prev_len);
    self.exit_cfg(cfg);
  }

  fn visit_field(&mut self, i: &'ast syn::Field) {
    let cfg = self.enter_cfg(&i.attrs);
    visit_field(self, i);
    self.exit_cfg(cfg);
  }

  fn visit_variant(&mut self, i: &'ast syn::Variant) {
    let cfg = self.enter_cfg(&i.attrs);
    visit_variant(self, i);
    self.exit_cfg(cfg);
  }

  fn visit_foreign_item(&mut self, i: &'ast syn::ForeignItem) {
    let attrs = match i {
      ForeignItem::Fn(f) => &f.attrs[..],
      ForeignItem::Static(s) => &s.attrs[..],
      syn::ForeignItem::Type(t) => &t.attrs[..],
      syn::ForeignItem::Macro(m) => &m.attrs[..],
      _ => &[],
    };
    let cfg = self.enter_cfg(attrs);
    visit_foreign_item(self, i);
    self.exit_cfg(cfg);
  }

  fn visit_item_use(&mut self, i: &'ast ItemUse) {
    self.use_depth += 1;
    visit_item_use(self, i);
    self.use_depth -= 1;
  }

  fn visit_attribute(&mut self, i: &'ast Attribute) {
    self.attr_depth += 1;
    visit_attribute(self, i);
    self.attr_depth -= 1;
  }

  fn visit_item_mod(&mut self, i: &'ast ItemMod) {
    if let Some((_, ref items)) = i.content {
      let (insert_pos, indent, has_existing_use) =
        find_mod_insert_pos_and_indent(i, items, self.source, self.line_offsets);
      let child = if let Some(parent) = self.module_stack.last() {
        child_scope(&parent.scope, items)
      } else {
        super::scope::collect_items_scope(items)
      };
      self.module_stack.push(ModuleContext {
        insert_pos,
        indent,
        has_existing_use,
        scope: child,
        paths: Vec::new(),
      });

      visit_item_mod(self, i);

      if let Some(m) = self.module_stack.pop() {
        self.modules.push(m);
      }
    } else {
      visit_item_mod(self, i);
    }
  }

  fn visit_item_macro(&mut self, _i: &'ast syn::ItemMacro) {
    // 宏定义（macro_rules!）内部绝对不替换，直接跳过！
  }

  fn visit_macro(&mut self, i: &'ast Macro) {
    // 宏调用处（如 format!(...), println!(...), vec![...] 等）
    if self.use_depth == 0 && self.attr_depth == 0 {
      self.inspect_path(&i.path);

      let parser = Punctuated::<syn::Expr, Token![,]>::parse_terminated;
      if let Ok(exprs) = parser.parse2(i.tokens.clone()) {
        for expr in &exprs {
          self.visit_expr(expr);
        }
      }
    }
  }

  fn visit_expr_path(&mut self, i: &'ast syn::ExprPath) {
    for attr in &i.attrs {
      self.visit_attribute(attr);
    }
    if let Some(ref qself) = i.qself {
      self.visit_type(&qself.ty);
      if self.inspect_qself_trait(qself, &i.path) {
        return;
      }
    }
    self.visit_path(&i.path);
  }

  fn visit_type_path(&mut self, i: &'ast syn::TypePath) {
    if let Some(ref qself) = i.qself {
      self.visit_type(&qself.ty);
      if self.inspect_qself_trait(qself, &i.path) {
        return;
      }
    }
    self.visit_path(&i.path);
  }

  fn visit_path(&mut self, p: &'ast Path) {
    if self.use_depth == 0 && self.attr_depth == 0 {
      self.inspect_path(p);
    }
    visit_path(self, p);
  }
}

impl PathCollector<'_> {
  fn inspect_qself_trait(&mut self, qself: &syn::QSelf, path: &Path) -> bool {
    if qself.as_token.is_some()
      && qself.position > 0
      && qself.position <= path.segments.len()
      && self.use_depth == 0
      && self.attr_depth == 0
    {
      let trait_segments = path.segments.iter().take(qself.position).cloned().collect();
      let trait_path = Path {
        leading_colon: path.leading_colon,
        segments: trait_segments,
      };
      self.inspect_path(&trait_path);
      return true;
    }
    qself.as_token.is_some() && qself.position > 0 && qself.position <= path.segments.len()
  }

  #[inline]
  fn enter_cfg(&mut self, attrs: &[Attribute]) -> bool {
    if has_cfg(attrs) {
      self.cfg_depth += 1;
      true
    } else {
      false
    }
  }

  #[inline]
  fn exit_cfg(&mut self, entered: bool) {
    if entered {
      self.cfg_depth -= 1;
    }
  }
}

fn collect_pat_idents(pat: &syn::Pat, out: &mut Vec<String>) {
  match pat {
    Pat::Ident(i) => {
      out.push(i.ident.to_string());
    }
    syn::Pat::Tuple(t) => {
      for p in &t.elems {
        collect_pat_idents(p, out);
      }
    }
    Pat::TupleStruct(ts) => {
      for p in &ts.elems {
        collect_pat_idents(p, out);
      }
    }
    syn::Pat::Struct(s) => {
      for f in &s.fields {
        collect_pat_idents(&f.pat, out);
      }
    }
    Pat::Slice(s) => {
      for p in &s.elems {
        collect_pat_idents(p, out);
      }
    }
    syn::Pat::Reference(r) => {
      collect_pat_idents(&r.pat, out);
    }
    syn::Pat::Type(t) => {
      collect_pat_idents(&t.pat, out);
    }
    Pat::Or(o) => {
      for p in &o.cases {
        collect_pat_idents(p, out);
      }
    }
    Pat::Paren(p) => {
      collect_pat_idents(&p.pat, out);
    }
    _ => {}
  }
}

fn collect_sig_inputs(sig: &syn::Signature, out: &mut Vec<String>) {
  for input in &sig.inputs {
    if let FnArg::Typed(t) = input {
      collect_pat_idents(&t.pat, out);
    }
  }
}

fn collect_item_idents(item: &syn::Item, out: &mut Vec<String>) {
  let ident = match item {
    Item::Fn(f) => Some(&f.sig.ident),
    Item::Struct(s) => Some(&s.ident),
    Item::Enum(e) => Some(&e.ident),
    Item::Const(c) => Some(&c.ident),
    Item::Static(s) => Some(&s.ident),
    Item::Trait(t) => Some(&t.ident),
    Item::TraitAlias(ta) => Some(&ta.ident),
    Item::Type(t) => Some(&t.ident),
    Item::Union(u) => Some(&u.ident),
    Item::Mod(m) => Some(&m.ident),
    _ => None,
  };
  if let Some(ident) = ident {
    out.push(ident.to_string());
  }
}

#[inline]
fn has_cfg(attrs: &[Attribute]) -> bool {
  attrs.iter().any(|a| {
    let p = a.path();
    p.is_ident("cfg") || p.is_ident("cfg_attr")
  })
}

fn item_attrs(item: &Item) -> &[Attribute] {
  // 内嵌模块自带独立的模块作用域，use 语句就地插入其模块体内部，天然受到模块自身 cfg 保护，
  // 绝不会泄露至外层无条件作用域，因此不递增 cfg_depth
  if let Item::Mod(m) = item
    && m.content.is_some()
  {
    return &[];
  }
  match item {
    Item::Fn(i) => &i.attrs,
    Item::Struct(i) => &i.attrs,
    Item::Enum(i) => &i.attrs,
    Item::Impl(i) => &i.attrs,
    Item::Trait(i) => &i.attrs,
    Item::Type(i) => &i.attrs,
    Item::Const(i) => &i.attrs,
    Item::Static(i) => &i.attrs,
    Item::Mod(i) => &i.attrs,
    Item::ForeignMod(i) => &i.attrs,
    Item::Macro(i) => &i.attrs,
    Item::Union(i) => &i.attrs,
    Item::Use(i) => &i.attrs,
    _ => &[],
  }
}
