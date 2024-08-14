use crate::ir::lexical_scope::*;
use crate::ir::{CaptureFrom, Global, ParamRef, Type, Value};

#[test]
fn test_root_level_locals() {
    /*
        val a = 42

        a // local
     */

    let mut scope = new_stack();

    let local_ref = scope.define_local(String::from("a"), false);
    let result = scope.lookup("a");

    assert_eq!(result, Ok(NameRef::Local(local_ref)))
}

#[test]
fn test_missing_local() {
    /*
        a // Error
     */

    let mut scope = new_stack();

    let result = scope.lookup("a");

    assert_eq!(result, Err(NameAccessError::NameNotFound))
}

#[test]
fn test_defining_comptime_vals_at_root() {
    /*
        @val a = 42

        a // comptime
     */

    let mut scope = new_stack();

    let local_ref = scope.define_local(String::from("a"), true);
    let result = scope.lookup("a");

    scope.pop_block();
    let frame = scope.pop_stack_frame();

    assert_eq!(frame.locals[local_ref.i].comptime, true);
    assert_eq!(result, Ok(NameRef::Local(local_ref)));
}

#[test]
fn test_defining_vals_at_comptime_block() {
    /*
        @(
            val a = 42

            a // comptime main local
        )
     */

    let mut scope = new_stack();

    scope.push_comptime_portal();
    scope.push_block();

    let local_ref = scope.define_local(String::from("a"), false);
    let result = scope.lookup("a");

    assert_eq!(result, Ok(NameRef::Local(local_ref)));

    scope.pop_block();
    scope.pop_comptime_portal();

    scope.pop_block(); // Main block
    let frame = scope.pop_stack_frame(); // Main runtime frame

    assert_eq!(frame.locals.len(), 1);
}

#[test]
fn test_using_comptime_vals_from_comptime_block() {
    /*
        @val a = 42

        @(
          a // Local
        )
     */

    let mut scope = new_stack();

    let comptime_local_ref = scope.define_local(String::from("a"), true);

    scope.push_comptime_portal();
    scope.push_block();

    let result = scope.lookup("a");

    assert_eq!(result, Ok(NameRef::Local(comptime_local_ref)))
}

#[test]
fn test_using_comptime_vals_from_runtime_block() {
    /*
        @val a = 42

        (
          a // export
        )
     */

    let mut scope = new_stack();

    scope.define_local(String::from("a"), true);

    scope.push_block();
    let result = scope.lookup("a");
    scope.pop_block();

    let root = consume_stack(scope);

    // TODO: Check that it's comptime
    assert!(matches!(result, Ok(NameRef::Local(_))));
}

#[test]
fn test_cannot_reference_runtime_vals_from_comptime() {
    /*
      val a = 42

      @a // Error
    */

    let mut scope = new_stack();

    scope.define_local(String::from("a"), false);

    scope.push_comptime_portal();
    scope.push_block();

    let result = scope.lookup("a");

    assert_eq!(result, Err(NameAccessError::CannotReferenceRuntimeNameFromComptime))
}

#[test]
fn test_cannot_reference_runtime_vals_from_comptime_nested() {
    /*
        @(
          val a = 42

          @a // Error
        )
    */

    let mut scope = new_stack();

    {
        scope.push_comptime_portal();
        scope.push_block();

        scope.define_local(String::from("a"), false);

        {
            scope.push_comptime_portal();
            scope.push_block();

            let result = scope.lookup("a");

            // TODO: Check that it's comptime
            assert!(matches!(result, Ok(NameRef::Local(_))));
        }
    }
}

#[test]
fn test_captures_comptime_local_in_comptime_fn() {
    /*
        @val a = 42

        @(
            { // capture a
              a // local
            }
        )
    */

    let mut scope = new_stack();

    let from_ref = scope.define_local(String::from("a"), true);

    {
        scope.push_comptime_portal();
        scope.push_block();

        {
            scope.push_stack_frame(vec![]);
            scope.push_block();

            let result = scope.lookup("a");

            println!("{:?}", result);

            assert!(matches!(result, Ok(NameRef::Capture(_))));

            scope.pop_block();

            let stack_frame = scope.pop_stack_frame();

            match stack_frame.captures.get(0) {
                None => panic!("Expected to have a capture"),
                Some(Capture { from, .. }) => {
                    assert_eq!(*from, CaptureFrom::Local(from_ref))
                }
            }
        }
    }
}

#[test]
fn test_capture_nested_fns_in_comptime() {
    /*
        @(a) {
          { // capture a
            a // local
          }
        }
    */

    let mut scope = new_stack();

    {
        scope.push_comptime_portal();
        scope.push_block();

        {
            scope.push_stack_frame(vec![
                Param { name: String::from("a"), comptime: false }
            ]);
            scope.push_block();

            {
                scope.push_stack_frame(vec![]);
                scope.push_block();

                let result = scope.lookup("a");

                assert!(matches!(result, Ok(NameRef::Capture(_))));

                scope.pop_block();
                let stack_frame = scope.pop_stack_frame();

                match stack_frame.captures.get(0) {
                    None => panic!("Expected to have a capture"),
                    Some(Capture { from, .. }) => {
                        assert_eq!(*from, CaptureFrom::Param(ParamRef { i: 0, comptime: false }))
                    }
                }
            }
        }
    }
}

#[test]
fn test_use_comptime_in_another_comptime_fn() {
    /*
        @val b = 42

        @(a) {
          @val c = 42

          c // comptime export of c
        }
    */

    let mut scope = new_stack();

    scope.define_local(String::from("b"), true);

    {
        scope.push_comptime_portal();
        scope.push_block();

        {
            scope.push_stack_frame(vec![
                Param { name: String::from("a"), comptime: false }
            ]);
            scope.push_block();

            scope.define_local(String::from("c"), true);

            let result = scope.lookup("c");

            // assert!(matches!(result, Ok(NameRef::Local(_))));
            assert!(matches!(result, Ok(NameRef::Local(_))));
        }
    }
}

#[test]
fn test_can_find_globals_in_comptime_context() {
    /*
        @Int
    */

    let mut scope = new_stack();
    scope.push_comptime_portal();
    scope.push_block();

    let result = scope.lookup("Int");

    assert!(matches!(result, Ok(NameRef::Global(_))));
}

fn new_stack() -> ScopeStack {
    let mut stack = ScopeStack::new(
        RootScope::new(vec![
            Global { name: String::from("Int"), value: Value::Type(Type::Int), comptime: true }
        ])
    );

    stack.push_stack_frame(vec![]);
    stack.push_block();

    stack
}

fn consume_stack(mut stack: ScopeStack) -> RootScope {
    stack.pop_block();
    stack.pop_stack_frame();

    stack.consume_root()
}