use allay_base::dependency::{DependencyError, DependencyGraph};

#[test]
fn test_basic_dependency() {
    let mut graph = DependencyGraph::default();
    // Add dependencies
    assert!(graph.add_dependency("A", "B").is_ok());
    assert!(graph.add_dependency("B", "C").is_ok());
    assert!(graph.add_dependency("A", "C").is_ok());

    // Check direct dependencies
    assert!(graph.is_dependent(&"A", &"B"));
    assert!(graph.is_dependent(&"B", &"C"));
    assert!(graph.is_dependent(&"A", &"C"));
    assert!(!graph.is_dependent(&"C", &"A"));

    // Check depends_on and depended_by
    assert_eq!(graph.depends_on(&"C").unwrap().len(), 2);
    assert_eq!(graph.depended_by(&"A").unwrap().len(), 2);
}

#[test]
fn test_recursive_dependency() {
    let mut graph = DependencyGraph::default();
    // Add dependencies
    assert!(graph.add_dependency("A", "B").is_ok());
    assert!(graph.add_dependency("B", "C").is_ok());
    assert!(graph.add_dependency("C", "D").is_ok());

    // Check leaf and root
    assert!(graph.is_leaf(&"A"));
    assert!(graph.is_root(&"D"));

    // Check recursive dependencies
    assert!(graph.is_dependent_recursively(&"A", &"D"));
    assert!(graph.is_dependent_recursively(&"B", &"D"));
    assert!(!graph.is_dependent_recursively(&"D", &"A"));

    // Check depends_on and depended_by
    assert_eq!(graph.depends_on_recursively(&"D").len(), 3);
    assert_eq!(graph.depended_by_recursively(&"A").len(), 3);
}

#[test]
fn test_dependency_errors() {
    let mut graph = DependencyGraph::default();
    // Add a dependency
    assert!(graph.add_dependency("A", "B").is_ok());

    assert!(matches!(
        graph.add_dependency("A", "A").unwrap_err(),
        DependencyError::SelfDependency
    ));

    assert!(matches!(
        graph.add_dependency("B", "A").unwrap_err(),
        DependencyError::CircularDependency
    ));

    assert!(matches!(
        graph.replace_point(&"A", &"B").unwrap_err(),
        DependencyError::ReplaceToExistingPoint
    ));
}

#[test]
fn test_remove_and_replace_dependency() {
    let mut graph = DependencyGraph::default();
    // Add dependencies
    assert!(graph.add_dependency("A", "D").is_ok());
    assert!(graph.add_dependency("D", "C").is_ok());

    // Replace a point
    assert!(graph.replace_point(&"D", &"B").is_ok());

    // Remove a dependency
    graph.remove_dependency(&"A", &"B");
    assert!(!graph.is_dependent(&"A", &"B"));
    assert!(graph.is_dependent(&"B", &"C"));

    // Remove a point
    graph.remove_point(&"B");
    assert!(!graph.exists(&"B"));
    assert!(!graph.is_dependent(&"A", &"C"));
}

#[test]
fn test_complex_dependency_scenarios() {
    let mut graph = DependencyGraph::default();

    //     A   B   C
    //    / \ / \ / \
    //   D   E   F   G
    //    \ / \ / \ /
    //     H   I   J
    //      \ / \ /
    //       K   L
    //        \ /
    //         M

    // layer 1
    assert!(graph.add_dependency("D", "A").is_ok());
    assert!(graph.add_dependency("E", "A").is_ok());
    assert!(graph.add_dependency("E", "B").is_ok());
    assert!(graph.add_dependency("F", "B").is_ok());
    assert!(graph.add_dependency("F", "C").is_ok());
    assert!(graph.add_dependency("G", "C").is_ok());

    // layer 2
    assert!(graph.add_dependency("H", "D").is_ok());
    assert!(graph.add_dependency("H", "E").is_ok());
    assert!(graph.add_dependency("I", "E").is_ok());
    assert!(graph.add_dependency("I", "F").is_ok());
    assert!(graph.add_dependency("J", "F").is_ok());
    assert!(graph.add_dependency("J", "G").is_ok());

    // layer 3
    assert!(graph.add_dependency("K", "H").is_ok());
    assert!(graph.add_dependency("K", "I").is_ok());
    assert!(graph.add_dependency("L", "I").is_ok());
    assert!(graph.add_dependency("L", "J").is_ok());

    // layer 4
    assert!(graph.add_dependency("M", "K").is_ok());
    assert!(graph.add_dependency("M", "L").is_ok());

    // direct dependencies
    assert!(graph.is_dependent(&"D", &"A"));
    assert!(graph.is_dependent(&"E", &"A"));
    assert!(graph.is_dependent(&"E", &"B"));
    assert!(graph.is_dependent(&"M", &"K"));
    assert!(graph.is_dependent(&"M", &"L"));

    // recursive dependencies
    assert!(graph.is_dependent_recursively(&"M", &"A"));
    assert!(!graph.is_dependent_recursively(&"I", &"G"));

    // root and leaf checks
    assert!(graph.is_root(&"A"));
    assert!(graph.is_root(&"B"));
    assert!(graph.is_root(&"C"));
    assert!(!graph.is_root(&"G"));

    assert!(graph.is_leaf(&"M"));
    assert!(!graph.is_leaf(&"H"));

    let a_dependents = graph.depends_on_recursively(&"A");
    assert_eq!(a_dependents.len(), 7); // D, E, H, I, K, L, M
    assert!(a_dependents.contains("D"));
    assert!(a_dependents.contains("H"));

    let m_dependencies = graph.depended_by_recursively(&"M");
    assert_eq!(m_dependencies.len(), 12);

    // error checks
    assert!(matches!(
        graph.add_dependency("A", "M").unwrap_err(),
        DependencyError::CircularDependency
    ));

    assert!(matches!(
        graph.add_dependency("B", "I").unwrap_err(),
        DependencyError::CircularDependency
    ));

    assert!(matches!(
        graph.add_dependency("X", "X").unwrap_err(),
        DependencyError::SelfDependency
    ));

    // replace point test
    assert!(graph.replace_point(&"K", &"K2").is_ok());
    assert!(!graph.exists(&"K"));
    assert!(graph.exists(&"K2"));
    assert!(graph.is_dependent(&"M", &"K2"));
    assert!(graph.is_dependent(&"K2", &"H"));
    assert!(graph.is_dependent(&"K2", &"I"));

    assert!(matches!(
        graph.replace_point(&"K2", &"M").unwrap_err(),
        DependencyError::ReplaceToExistingPoint
    ));

    graph.remove_dependency(&"M", &"K2");
    assert!(!graph.is_dependent(&"M", &"K2"));
    assert!(graph.is_dependent(&"M", &"L"));

    graph.remove_point(&"M");
    assert!(!graph.exists(&"M"));

    assert!(graph.is_dependent(&"K2", &"H"));
    assert!(graph.is_dependent(&"K2", &"I"));
    assert!(graph.is_dependent(&"L", &"I"));
    assert!(graph.is_dependent(&"L", &"J"));

    let empty_graph = DependencyGraph::<&str>::default();
    assert!(!empty_graph.exists(&"A"));
    assert!(empty_graph.is_root(&"A"));
    assert!(empty_graph.is_leaf(&"A"));

    assert!(graph.add_dependency("K2", "H").is_ok());
    assert!(graph.add_dependency("K2", "H").is_ok());

    assert_eq!(graph.depends_on(&"A").unwrap().len(), 2); // D, E
    assert_eq!(graph.depended_by(&"D").unwrap().len(), 1); // A
    assert_eq!(graph.depended_by(&"E").unwrap().len(), 2); // A, B
}
