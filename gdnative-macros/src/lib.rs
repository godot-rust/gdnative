#![feature(proc_macro)]
#![recursion_limit = "128"]
extern crate proc_macro;
//extern crate syn;
extern crate syntex;
extern crate syntex_syntax;
#[macro_use] extern crate quote;

use proc_macro::TokenStream;

use syntex_syntax::print::pprust::*;

#[proc_macro_attribute]
pub fn godot_export(_: TokenStream, input: TokenStream) -> TokenStream {
  let parse_sess = syntex_syntax::parse::ParseSess::new();

  let ast = syntex_syntax::parse::parse_item_from_source_str(String::new(), input.to_string(), &parse_sess).unwrap().unwrap();

  match &ast.node {
    &syntex_syntax::ast::ItemKind::Impl(_, _, _, _, ref ty, ref implitems) => {
      
      let ty = quote::Ident::from(ty_to_string(ty));

      let export_items = implitems.iter()
        .filter(|&item| { if let syntex_syntax::ast::Visibility::Public = item.vis {true} else {false} })
        .filter(|&item| { if let syntex_syntax::ast::ImplItemKind::Method(_,_) = item.node {true} else {false} })
        .map(|item| { if let syntex_syntax::ast::ImplItemKind::Method(ref signature, _) = item.node {(item.ident, signature.decl.clone())} else {unreachable!()} }).collect::<Vec<_>>();


      let mut gen = quote::Tokens::new();

      for item in &export_items {
        if item.1.inputs.len() == 0 {
          gen.append(impl_godot_export(format!("{}", item.0).as_str(), None, &Vec::new()).as_str());
        } else if item.1.inputs[0].is_self() {
          let (self_arg, args) = item.1.inputs.split_first().unwrap();
          gen.append(impl_godot_export(format!("{}", item.0).as_str(), Some(self_arg), &args.to_vec()).as_str());
        } else {
          gen.append(impl_godot_export(format!("{}", item.0).as_str(), None, &item.1.inputs).as_str());
        }
      }

      let register_methods = export_items.iter()
        .map(|i| (ident_to_string(i.0)+"_godot_wrapper", ident_to_string(i.0)))
        .fold(quote::Tokens::new(), |mut t, i| {
          let function_name = quote::Ident::new(i.1);
          let wrapper_name = quote::Ident::new(i.0);
          t.append(
            quote!{
              gdnative::gdnative_sys::godot_script_register_method(
                std::ffi::CString::new(stringify!(#ty)).unwrap().as_ptr(),
                std::ffi::CString::new(stringify!(#function_name)).unwrap().as_ptr(),
                &mut gdnative::gdnative_sys::godot_method_attributes{rpc_type: gdnative::gdnative_sys::GODOT_METHOD_RPC_MODE_DISABLED as i32} as *mut gdnative::gdnative_sys::godot_method_attributes, //TODO
                gdnative::gdnative_sys::godot_instance_method{
                  method: Some(#ty::#wrapper_name),
                  method_data: std::ptr::null_mut(),
                  free_func: None
                },
              );
            }.as_str());
          t
        });

      ("#[allow(warnings)]".to_string() + &(input.to_string() + 
        &("#[allow(warnings)]".to_string() + quote!{
          impl #ty {
            #gen
          }
          impl gdnative::GodotObjectRegisterMethods for #ty {
            fn register_methods() {
              unsafe {
                #register_methods
              }
            }
          }
        }.as_str()))).replace("#[godot_export]", "")
      .parse().unwrap()
    },
      
    _ => panic!("not an impl block!")
  }
}

fn impl_godot_export(function_name: &str, self_arg: Option<&syntex_syntax::ast::Arg>, function_args: &Vec<syntex_syntax::ast::Arg>) -> quote::Tokens {
  let wrapper_name = quote::Ident::new(function_name.to_owned() + "_godot_wrapper");
  let function_name_ident = quote::Ident::new(function_name);
  let num_args = function_args.len() as i32;

  let mut args = quote::Tokens::new();
  if let Some(arg) = self_arg {
    args.append(&format!("{},", arg_to_string(&arg)).replace("self", "this"));
  };
  
  let args: quote::Tokens = (0..function_args.len()).fold(args, |mut t, i| {
    t.append(&format!("(*(*p_args.offset({}))).into(),", i));
    t
  });
  quote! {
    pub unsafe extern "C" fn #wrapper_name(
    p_instance: *mut gdnative::gdnative_sys::godot_object,
    p_method_data: *mut std::os::raw::c_void,
    p_data: *mut std::os::raw::c_void,
    p_args: *mut *mut gdnative::gdnative_sys::godot_variant,
    p_num_args: std::os::ffi::) -> gdnative::gdnative_sys::godot_variant {
      #![allow(unused_mut)]
      #![allow(unused_variables)]
      use std::panic::catch_unwind;

      let mut this: *mut Self = p_data as *mut Self;
      let mut this = std::boxed::Box::from_raw(this);
      
      if p_num_args != #num_args {
        println!("wrong number of arguments");
        std::process::exit(1);
      };

      let ret_val = if let Ok(r) = catch_unwind(std::panic::AssertUnwindSafe(|| {
        Self::#function_name_ident(#args)
      })) {
        r
      } else {
        println!("function {} panicked!", #function_name);
        std::process::exit(1);
      };

      std::mem::forget(this);

      ret_val.into()
    }
  }
}
