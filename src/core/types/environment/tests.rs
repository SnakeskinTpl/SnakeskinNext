use super::{Type, TypeEnvironment};

#[test]
fn scope_exit_restores_shadowed_bindings_and_mutability() {
    let mut environment = TypeEnvironment::new();
    assert!(environment.declare("value", Type::Bool, false));

    environment.push_scope();
    assert!(environment.declare("value", Type::Int { bits: 64 }, true));

    assert_eq!(environment.get("value"), Some(&Type::Int { bits: 64 }));
    assert!(environment.binding("value").unwrap().mutable);
    assert!(!environment.declare("value", Type::Str, false));
    assert_eq!(environment.get("value"), Some(&Type::Int { bits: 64 }));

    environment.pop_scope();

    assert_eq!(environment.get("value"), Some(&Type::Bool));
    assert!(!environment.binding("value").unwrap().mutable);
    assert_eq!(environment.len(), 1);
}
