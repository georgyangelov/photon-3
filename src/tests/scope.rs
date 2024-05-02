use crate::compiler::scope::{BlockScope, ComptimeMainStackFrame, ComptimePortal, NameAccessError, NameRef, RootScope, StackFrame};

#[test]
fn test_root_level_locals() {
    /*
        val a = 42

        a // local
     */

    let mut root = RootScope::new();
    let mut comptime_main = ComptimeMainStackFrame::new(&mut root);
    let mut block = BlockScope::new(&mut comptime_main);

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
    let mut comptime_main = ComptimeMainStackFrame::new(&mut root);
    let mut block = BlockScope::new(&mut comptime_main);

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
    let mut comptime_main = ComptimeMainStackFrame::new(&mut root);
    let mut block = BlockScope::new(&mut comptime_main);

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
    let mut comptime_main = ComptimeMainStackFrame::new(&mut root);
    let mut block = BlockScope::new(&mut comptime_main);

    let comptime_local_ref = block.define_comptime_main_local(String::from("a"));

    let mut comptime_portal = ComptimePortal::new(&mut block);
    let mut block = BlockScope::new(&mut comptime_portal);

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
    let mut comptime_main = ComptimeMainStackFrame::new(&mut root);
    let mut block = BlockScope::new(&mut comptime_main);

    block.define_comptime_main_local(String::from("a"));

    let mut block = BlockScope::new(&mut block);

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
    let mut comptime_main = ComptimeMainStackFrame::new(&mut root);
    let mut block = BlockScope::new(&mut comptime_main);

    block.define_comptime_main_local(String::from("a"));

    let result_1 = block.access_local("a");

    let mut block = BlockScope::new(&mut block);

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
    let mut comptime_main = ComptimeMainStackFrame::new(&mut root);
    let mut block = BlockScope::new(&mut comptime_main);

    block.define_local(String::from("a"));

    let mut comptime_portal = ComptimePortal::new(&mut block);
    let mut block = BlockScope::new(&mut comptime_portal);

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
    let mut comptime_main = ComptimeMainStackFrame::new(&mut root);
    let mut block = BlockScope::new(&mut comptime_main);

    {
        let mut comptime_portal = ComptimePortal::new(&mut block);
        let mut block = BlockScope::new(&mut comptime_portal);

        block.define_local(String::from("a"));

        {
            let mut comptime_portal = ComptimePortal::new(&mut block);
            let mut block = BlockScope::new(&mut comptime_portal);

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
    let mut comptime_main = ComptimeMainStackFrame::new(&mut root);
    let mut block = BlockScope::new(&mut comptime_main);

    let from_ref = block.define_comptime_main_local(String::from("a"));

    {
        let mut comptime_portal = ComptimePortal::new(&mut block);
        let mut block = BlockScope::new(&mut comptime_portal);

        {
            let mut stack_frame = StackFrame::new(&mut block);
            let mut block = BlockScope::new(&mut stack_frame);

            let result = block.access_local("a");

            assert!(matches!(result, Ok(NameRef::Local(_))));
            assert_eq!(stack_frame.captures.len(), 1);
            // assert_eq!(stack_frame.captures.get(0), Some());
        }
    }
}

/*
    @(a) {
      a // local
    }
*/

/*
    @(a) {
      { // capture a
        a // local
      }
    }
*/

/*
    @val b = 42

    @(a) { // capture b
      a // local
      b // local
    }
*/

/*
    @val b = 42

    @(a) {
      @val c = 42

      c // error
    }
*/