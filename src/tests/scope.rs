use crate::compiler::lexical_scope::*;

#[test]
fn test_root_level_locals() {
    /*
        val a = 42

        a // local
     */

    let mut scope = new_stack();

    let local_ref = scope.define_local(String::from("a"));
    let result = scope.access_local("a");

    assert_eq!(result, Ok(AccessNameRef::Local(local_ref)))
}

#[test]
fn test_missing_local() {
    /*
        a // Error
     */

    let mut scope = new_stack();

    let result = scope.access_local("a");

    assert_eq!(result, Err(NameAccessError::NameNotFound))
}

#[test]
fn test_defining_comptime_vals_at_root() {
    /*
        @val a = 42

        a // comptime export
     */

    let mut scope = new_stack();

    scope.define_comptime_main_local(String::from("a"));

    let result = scope.access_local("a");

    assert!(matches!(result, Ok(AccessNameRef::ComptimeExport(_, Some(_)))))
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

    let local_ref = scope.define_local(String::from("a"));
    let result = scope.access_local("a");

    assert_eq!(result, Ok(AccessNameRef::Local(local_ref)));

    scope.pop_block();
    scope.pop_comptime_portal();

    scope.pop_block(); // Main block
    let runtime_frame = scope.pop_stack_frame(); // Main runtime frame
    let comptime_frame = scope.pop_comptime_main_stack_frame();

    assert_eq!(runtime_frame.locals.len(), 0);
    assert_eq!(comptime_frame.locals.len(), 1);
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

    let comptime_local_ref = scope.define_comptime_main_local(String::from("a"));

    scope.push_comptime_portal();
    scope.push_block();

    let result = scope.access_local("a");

    assert_eq!(result, Ok(AccessNameRef::Local(comptime_local_ref)))
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

    scope.define_comptime_main_local(String::from("a"));

    scope.push_block();

    let result = scope.access_local("a");

    assert!(matches!(result, Ok(AccessNameRef::ComptimeExport(_, Some(_)))))
}

#[test]
fn test_reuses_comptime_slots() {
    /*
        @val a = 42

        a // export

        (
          a // same exact export slot
        )
     */

    let mut scope = new_stack();

    scope.define_comptime_main_local(String::from("a"));

    let result_1 = scope.access_local("a").unwrap();
    assert!(matches!(result_1, AccessNameRef::ComptimeExport(_, Some(_))));

    scope.push_block();

    let result_2 = scope.access_local("a").unwrap();
    assert!(matches!(result_2, AccessNameRef::ComptimeExport(_, None)));

    let ref_1 = match result_1 {
        AccessNameRef::ComptimeExport(ref_1, _) => ref_1,
        _ => panic!("Invalid result type")
    };

    let ref_2 = match result_2 {
        AccessNameRef::ComptimeExport(ref_2, _) => ref_2,
        _ => panic!("Invalid result type")
    };

    assert_eq!(ref_1, ref_2)
}

#[test]
fn test_cannot_reference_runtime_vals_from_comptime() {
    /*
      val a = 42

      @a // Error
    */

    let mut scope = new_stack();

    scope.define_local(String::from("a"));

    scope.push_comptime_portal();
    scope.push_block();

    let result = scope.access_local("a");

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

        scope.define_local(String::from("a"));

        {
            scope.push_comptime_portal();
            scope.push_block();

            let result = scope.access_local("a");

            assert_eq!(result, Err(NameAccessError::CannotReferenceRuntimeNameFromComptime))
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

    let from_ref = scope.define_comptime_main_local(String::from("a"));

    {
        scope.push_comptime_portal();
        scope.push_block();

        {
            scope.push_stack_frame();
            scope.push_block();

            let result = scope.access_local("a");

            assert!(matches!(result, Ok(AccessNameRef::Local(_))));

            scope.pop_block();

            let stack_frame = scope.pop_stack_frame();

            match stack_frame.captures.get(0) {
                None => panic!("Expected to have a capture"),
                Some(Capture { from, .. }) => {
                    assert_eq!(*from, from_ref)
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
            scope.push_stack_frame();
            scope.push_block();

            let from_ref = scope.define_local(String::from("a"));

            {
                scope.push_stack_frame();
                scope.push_block();

                let result = scope.access_local("a");

                assert!(matches!(result, Ok(AccessNameRef::Local(_))));

                scope.pop_block();
                let stack_frame = scope.pop_stack_frame();

                match stack_frame.captures.get(0) {
                    None => panic!("Expected to have a capture"),
                    Some(Capture { from, .. }) => {
                        assert_eq!(*from, from_ref)
                    }
                }
            }
        }
    }
}

#[test]
fn test_use_comptime_in_another_comptime_fn() {
    // This can't be a capture because c doesn't have a name outside that can be captured, although
    // captures don't need to have names, so... it can probably be a capture, but what if there are
    // multiple levels of closures?
    //
    // Actually this should be fine
    /*
        @val b = 42

        @(a) {
          @val c = 42

          c // comptime export of c
        }
    */

    let mut scope = new_stack();

    scope.define_comptime_main_local(String::from("b"));

    {
        scope.push_comptime_portal();
        scope.push_block();

        {
            scope.push_stack_frame();
            scope.push_block();

            scope.define_local(String::from("a"));

            {
                scope.push_stack_frame();
                scope.push_block();

                scope.define_comptime_main_local(String::from("c"));

                let result = scope.access_local("c");

                // assert!(matches!(result, Ok(NameRef::Local(_))));
                assert!(matches!(result, Ok(AccessNameRef::ComptimeExport(_, Some(_)))));
            }
        }
    }
}

fn new_stack() -> ScopeStack {
    let mut stack = ScopeStack::new(
        RootScope::new(),
        ComptimeMainStackFrame::new()
    );

    stack.push_stack_frame();
    stack.push_block();

    stack
}