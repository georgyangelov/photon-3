use crate::compiler::lexical_scope::*;

#[test]
fn test_root_level_locals() {
    /*
        val a = 42

        a // local
     */

    let mut root = RootScope::new();
    let mut comptime_main = root.new_comptime_main_frame();
    let mut runtime_main = comptime_main.new_runtime_main_frame();
    let mut block = runtime_main.new_child_block();

    let local_ref = block.define_local(String::from("a"));
    let result = block.access_local("a");

    assert_eq!(result, Ok(NameRef::Local(local_ref)))
}

#[test]
fn test_missing_local() {
    /*
        a // Error
     */

    let mut root = RootScope::new();
    let mut comptime_main = root.new_comptime_main_frame();
    let mut runtime_main = comptime_main.new_runtime_main_frame();
    let mut block = runtime_main.new_child_block();

    let result = block.access_local("a");

    assert_eq!(result, Err(NameAccessError::NameNotFound))
}

#[test]
fn test_defining_comptime_vals_at_root() {
    /*
        @val a = 42

        a // comptime export
     */

    let mut root = RootScope::new();
    let mut comptime_main = root.new_comptime_main_frame();
    let mut runtime_main = comptime_main.new_runtime_main_frame();
    let mut block = runtime_main.new_child_block();

    block.define_comptime_main_local(String::from("a"));

    let result = block.access_local("a");

    assert!(matches!(result, Ok(NameRef::ComptimeExport(_))))
}

#[test]
fn test_using_comptime_vals_from_comptime_block() {
    /*
        @val a = 42

        @(
          a // Local
        )
     */

    let mut root = RootScope::new();
    let mut comptime_main = root.new_comptime_main_frame();
    let mut runtime_main = comptime_main.new_runtime_main_frame();
    let mut block = runtime_main.new_child_block();

    let comptime_local_ref = block.define_comptime_main_local(String::from("a"));

    let mut comptime_portal = block.new_child_comptime_portal();
    let mut block = comptime_portal.new_child_block();

    let result = block.access_local("a");

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

    let mut root = RootScope::new();
    let mut comptime_main = root.new_comptime_main_frame();
    let mut runtime_main = comptime_main.new_runtime_main_frame();
    let mut block = runtime_main.new_child_block();

    block.define_comptime_main_local(String::from("a"));

    let mut block = block.new_child_block();

    let result = block.access_local("a");

    assert!(matches!(result, Ok(NameRef::ComptimeExport(_))))
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

    let mut root = RootScope::new();
    let mut comptime_main = root.new_comptime_main_frame();
    let mut runtime_main = comptime_main.new_runtime_main_frame();
    let mut block = runtime_main.new_child_block();

    block.define_comptime_main_local(String::from("a"));

    let result_1 = block.access_local("a");

    let mut block = block.new_child_block();

    let result_2 = block.access_local("a");

    assert_eq!(result_1, result_2)
}

#[test]
fn test_cannot_reference_runtime_vals_from_comptime() {
    /*
      val a = 42

      @a // Error
    */

    let mut root = RootScope::new();
    let mut comptime_main = root.new_comptime_main_frame();
    let mut runtime_main = comptime_main.new_runtime_main_frame();
    let mut block = runtime_main.new_child_block();

    block.define_local(String::from("a"));

    let mut comptime_portal = block.new_child_comptime_portal();
    let mut block = comptime_portal.new_child_block();

    let result = block.access_local("a");

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

    let mut root = RootScope::new();
    let mut comptime_main = root.new_comptime_main_frame();
    let mut runtime_main = comptime_main.new_runtime_main_frame();
    let mut block = runtime_main.new_child_block();

    {
        let mut comptime_portal = block.new_child_comptime_portal();
        let mut block = comptime_portal.new_child_block();

        block.define_local(String::from("a"));

        {
            let mut comptime_portal = block.new_child_comptime_portal();
            let mut block = comptime_portal.new_child_block();

            let result = block.access_local("a");

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

    let mut root = RootScope::new();
    let mut comptime_main = root.new_comptime_main_frame();
    let mut runtime_main = comptime_main.new_runtime_main_frame();
    let mut block = runtime_main.new_child_block();

    let from_ref = block.define_comptime_main_local(String::from("a"));

    {
        let mut comptime_portal = block.new_child_comptime_portal();
        let mut block = comptime_portal.new_child_block();

        {
            let mut stack_frame = block.new_child_stack_frame();
            let mut block = stack_frame.new_child_block();

            let result = block.access_local("a");

            assert!(matches!(result, Ok(NameRef::Local(_))));

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

    let mut root = RootScope::new();
    let mut comptime_main = root.new_comptime_main_frame();
    let mut runtime_main = comptime_main.new_runtime_main_frame();
    let mut block = runtime_main.new_child_block();

    {
        let mut comptime_portal = block.new_child_comptime_portal();
        let mut block = comptime_portal.new_child_block();

        {
            let mut stack_frame = block.new_child_stack_frame();
            let mut block = stack_frame.new_child_block();

            let from_ref = block.define_local(String::from("a"));

            {
                let mut stack_frame = block.new_child_stack_frame();
                let mut block = stack_frame.new_child_block();

                let result = block.access_local("a");

                assert!(matches!(result, Ok(NameRef::Local(_))));

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

    let mut root = RootScope::new();
    let mut comptime_main = root.new_comptime_main_frame();
    let mut runtime_main = comptime_main.new_runtime_main_frame();
    let mut block = runtime_main.new_child_block();

    block.define_comptime_main_local(String::from("b"));

    {
        let mut comptime_portal = block.new_child_comptime_portal();
        let mut block = comptime_portal.new_child_block();

        {
            let mut stack_frame = block.new_child_stack_frame();
            let mut block = stack_frame.new_child_block();

            block.define_local(String::from("a"));

            {
                let mut stack_frame = block.new_child_stack_frame();
                let mut block = stack_frame.new_child_block();

                block.define_comptime_main_local(String::from("c"));

                let result = block.access_local("c");

                // assert!(matches!(result, Ok(NameRef::Local(_))));
                assert!(matches!(result, Ok(NameRef::ComptimeExport(_))));
            }
        }
    }
}
