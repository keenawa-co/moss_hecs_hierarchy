use std::collections::HashSet;

use moss_hecs::{Entity, Frame};
use moss_hecs_hierarchy::{
    Child, Hierarchy, HierarchyMut, HierarchyQuery, TreeBuilder, TreeBuilderClone,
};
use moss_hecs_schedule::{CommandBuffer, GenericWorld, SubWorldRef};

#[derive(Debug)]
struct Tree;

#[test]
fn basic() {
    let mut frame = Frame::default();
    let root = frame.spawn(("Root",));

    // Attaches the child to a parent, in this case `root`
    let child_count = 10;

    // Make sure Hierarchy is correct but don't care about order.
    let mut expected_children: HashSet<Entity> = HashSet::new();

    for i in 0..child_count {
        let child = frame.spawn((format!("Child {}", i),));
        expected_children.insert(child);
        frame.attach::<Tree>(child, root).unwrap();
    }

    for child in frame.children::<Tree>(root) {
        let name = frame.get::<&String>(child).unwrap();

        println!(
            "Child: {:?} {:?}; {:?}",
            child,
            *name,
            *frame.get::<&Child<Tree>>(child).unwrap()
        );

        if !expected_children.remove(&child) {
            panic!("Entity {:?} does not belong in hierarchy", child);
        }
    }

    if !expected_children.is_empty() {
        panic!("Not all children in hierarchy were visited")
    }
}

#[test]
fn reattach2() {
    // Root ---- Child 1
    //      ---- Child 2
    let mut frame = Frame::default();
    let root = frame.spawn(("Root",));
    let child1 = frame.spawn(("Child1",));
    let child2 = frame.spawn(("Child2",));
    frame.attach::<Tree>(child1, root).unwrap();
    frame.attach::<Tree>(child2, root).unwrap();

    frame.detach::<Tree>(child2).unwrap();
    frame.attach::<Tree>(child2, child1).unwrap();
    frame.detach::<Tree>(child2).unwrap();
    frame.attach::<Tree>(child2, root).unwrap();

    for e in frame.descendants_depth_first::<Tree>(root) {
        println!("{:?}", *frame.get::<&&str>(e).unwrap());
    }

    frame.detach::<Tree>(child2).unwrap();
    frame.attach::<Tree>(child2, child1).unwrap();
    for e in frame.descendants_depth_first::<Tree>(root) {
        println!("{:?}", *frame.get::<&&str>(e).unwrap());
    }
}

#[test]
fn ancestors() {
    let mut frame = Frame::default();
    let depth = 10;
    let root = frame.spawn((String::from("Root"),));

    let mut children = vec![root];

    for i in 1..depth {
        let child = frame.spawn((format!("Child {}", i),));
        frame.attach::<Tree>(child, children[i - 1]).unwrap();
        children.push(child);
    }

    assert_eq!(
        frame
            .ancestors::<Tree>(children.pop().unwrap())
            .map(|parent| {
                println!("{}", *frame.get::<&String>(parent).unwrap());
                parent
            })
            .collect::<Vec<_>>(),
        children.into_iter().rev().collect::<Vec<_>>()
    );
}

#[test]
fn detach() {
    // Root ---- Child 1
    //      ---- Child 2
    //           ------- Child 3
    //      ---- Child 4
    //      ---- Child 5

    let mut frame = Frame::default();
    let root = frame.spawn(("Root",));
    let child1 = frame.attach_new::<Tree, _>(root, ("Child1",)).unwrap();
    let child2 = frame.attach_new::<Tree, _>(root, ("Child2",)).unwrap();
    let _child3 = frame.attach_new::<Tree, _>(child2, ("Child3",)).unwrap();
    let child4 = frame.attach_new::<Tree, _>(root, ("Child4",)).unwrap();
    let child5 = frame.attach_new::<Tree, _>(root, ("Child5",)).unwrap();

    // Remove child2, and by extension child3
    frame.detach::<Tree>(child2).unwrap();

    let order = [child1, child4, child5];

    for child in frame.children::<Tree>(root) {
        println!(
            "{:?}, {:?}",
            *frame.get::<&&str>(child).unwrap(),
            *frame.get::<&Child<Tree>>(child).unwrap()
        );
    }

    assert_eq!(
        frame.children::<Tree>(root).collect::<Vec<_>>(),
        order.iter().cloned().collect::<Vec<_>>()
    );
}

#[test]
fn reattach() {
    // Root ---- Child 1
    //      ---- Child 2
    //           ------- Child 3
    //                   ------- Child 4

    let mut frame = Frame::default();
    let root = frame.spawn(("Root",));
    let child1 = frame.attach_new::<Tree, _>(root, ("Child1",)).unwrap();
    let child2 = frame.attach_new::<Tree, _>(root, ("Child2",)).unwrap();
    let _child3 = frame.attach_new::<Tree, _>(child2, ("Child3",)).unwrap();
    let child4 = frame.attach_new::<Tree, _>(root, ("Child4",)).unwrap();
    let child5 = frame.attach_new::<Tree, _>(root, ("Child5",)).unwrap();

    // Remove child2, and by extension child3
    frame.detach::<Tree>(child2).unwrap();

    // Reattach child2 and child3 under child4
    frame.attach::<Tree>(child2, child4).unwrap();

    let order = [child1, child4, child5];

    for child in frame.descendants_depth_first::<Tree>(root) {
        println!(
            "{:?}, {:?}",
            *frame.get::<&&str>(child).unwrap(),
            *frame.get::<&Child<Tree>>(child).unwrap()
        );
    }

    assert_eq!(
        frame.children::<Tree>(root).collect::<Vec<_>>(),
        order.iter().cloned().collect::<Vec<_>>()
    );
}

#[test]
fn despawn() {
    // Root ---- Child 1
    //      ---- Child 2
    //           ------- Child 3
    //      ---- Child 4
    //      ---- Child 5

    let mut frame = Frame::default();
    let root = frame.spawn(("Root",));
    let child1 = frame.attach_new::<Tree, _>(root, ("Child1",)).unwrap();
    let child2 = frame.attach_new::<Tree, _>(root, ("Child2",)).unwrap();
    let child3 = frame.attach_new::<Tree, _>(child2, ("Child3",)).unwrap();
    let child4 = frame.attach_new::<Tree, _>(root, ("Child4",)).unwrap();
    let child5 = frame.attach_new::<Tree, _>(root, ("Child5",)).unwrap();

    frame.despawn_all::<Tree>(child3);

    assert_eq!(
        frame
            .descendants_depth_first::<Tree>(root)
            .collect::<Vec<_>>(),
        vec![child1, child2, child4, child5]
    );
}

#[test]
fn dfs() {
    // Root ---- Child 1
    //      ---- Child 2
    //           ------- Child 3
    //                   ------- Child 4

    let mut frame = Frame::default();
    let root = frame.spawn(("Root",));
    let child1 = frame.attach_new::<Tree, _>(root, ("Child1",)).unwrap();
    let child2 = frame.attach_new::<Tree, _>(root, ("Child2",)).unwrap();
    let child3 = frame.attach_new::<Tree, _>(child2, ("Child3",)).unwrap();
    let child4 = frame.attach_new::<Tree, _>(child3, ("Child4",)).unwrap();

    let order = [child1, child2, child3, child4];

    for child in frame.descendants_depth_first::<Tree>(root) {
        println!("{:?}", *frame.get::<&&str>(child).unwrap());
    }

    assert_eq!(
        frame
            .descendants_depth_first::<Tree>(root)
            .collect::<Vec<_>>(),
        order.iter().cloned().collect::<Vec<_>>()
    );
}

#[test]
fn dfs_skip() {
    // Root ---- Child 1
    //      ---- Child 2
    //           ------- Child 3
    //                   ------- Child 4

    struct Skip;

    let mut frame = Frame::default();
    let root = frame.spawn(("Root",));
    let child1 = frame.attach_new::<Tree, _>(root, ("Child1",)).unwrap();
    let child2 = frame.attach_new::<Tree, _>(root, ("Child2",)).unwrap();
    let child3 = frame
        .attach_new::<Tree, _>(child2, ("Child3", Skip))
        .unwrap();
    let _child4 = frame.attach_new::<Tree, _>(child3, ("Child4",)).unwrap();

    let order = [child1, child2];

    for child in frame.visit::<Tree, _>(root, |w, e| w.try_get::<Skip>(e).is_err()) {
        println!("{:?}", *frame.get::<&&str>(child).unwrap());
    }

    assert_eq!(
        frame
            .visit::<Tree, _>(root, |w, e| w.try_get::<Skip>(e).is_err())
            .collect::<Vec<_>>(),
        order.iter().cloned().collect::<Vec<_>>()
    );
}

#[test]
fn bfs() {
    // Root ---- Child 1
    //      ---- Child 2
    //           ------- Child 3
    //                   ------- Child 4

    let mut frame = Frame::default();
    let root = frame.spawn(("Root",));
    let child1 = frame.attach_new::<Tree, _>(root, ("Child1",)).unwrap();
    let child2 = frame.attach_new::<Tree, _>(root, ("Child2",)).unwrap();
    let child3 = frame.attach_new::<Tree, _>(child2, ("Child3",)).unwrap();
    let child4 = frame.attach_new::<Tree, _>(child3, ("Child4",)).unwrap();

    let order = [child1, child2, child3, child4];

    for child in frame.descendants_breadth_first::<Tree>(root) {
        println!("{:?}", *frame.get::<&&str>(child).unwrap());
    }

    assert_eq!(
        frame
            .descendants_breadth_first::<Tree>(root)
            .collect::<Vec<_>>(),
        order.iter().cloned().collect::<Vec<_>>()
    );
}

#[test]
fn empty() {
    let mut frame = Frame::default();

    let empty_root = frame.spawn(("Root",));

    assert_eq!(
        frame
            .children::<Tree>(empty_root)
            .map(|child| println!("Entity {:?} does not belong in hierarchy", child))
            .count(),
        0
    )
}

#[test]
fn roots() {
    let mut frame = Frame::default();
    let root1 = frame.spawn(("Root1",));
    let root2 = frame.spawn(("Root2",));
    let root3 = frame.spawn(("Root3",));

    frame.attach_new::<Tree, _>(root1, ("Child1",)).unwrap();
    frame.attach_new::<Tree, _>(root1, ("Child2",)).unwrap();
    frame.attach_new::<Tree, _>(root2, ("Child3",)).unwrap();
    frame.attach_new::<Tree, _>(root1, ("Child4",)).unwrap();
    frame.attach_new::<Tree, _>(root3, ("Child5",)).unwrap();

    let mut expected = [root1, root2, root3];
    expected.sort();

    let subframe = SubWorldRef::<HierarchyQuery<Tree>>::new(&frame);

    let mut roots = subframe
        .roots::<Tree>()
        .unwrap()
        .iter()
        .map(|(e, _)| e)
        .collect::<Vec<_>>();

    roots.sort();

    dbg!(&roots, &expected);

    assert_eq!(
        roots.iter().collect::<Vec<_>>(),
        expected.iter().collect::<Vec<_>>()
    );
}

#[test]
fn builder() {
    let mut frame = Frame::default();
    let mut builder = TreeBuilder::<Tree>::new();

    let root = builder
        .add("root")
        .attach(("child 1",))
        .attach(("child 2",))
        .attach_tree(
            TreeBuilder::from(("child 3",))
                .attach_move(("child 3.1",))
                .attach_move(("child 3.2",)),
        )
        .add_all(5.0_f32)
        .spawn(&mut frame);

    let expected = ["child 1", "child 2", "child 3", "child 3.1", "child 3.2"];

    assert!(frame
        .descendants_breadth_first::<Tree>(root)
        .zip(expected)
        .map(|(e, expected)| {
            let name = *frame.get::<&&str>(e).unwrap();
            eprintln!("Name: {}", name);

            let val = *frame.get::<&f32>(e).unwrap();
            name == expected && val == 5.0
        })
        .all(|val| val == true));
}

#[test]
fn builder_clone_deferred() {
    let mut frame = Frame::default();
    let mut cmd = CommandBuffer::new();

    let root = TreeBuilderClone::<Tree>::new()
        .add(("root",))
        .attach(("child 1",))
        .attach(("child 2",))
        .attach_tree(
            TreeBuilderClone::from(("child 3",))
                .attach_move(("child 3.1",))
                .attach_move(("child 3.2",)),
        )
        .clone() // Demonstrate cloning
        .spawn_deferred(&frame, &mut cmd);

    cmd.execute(&mut frame);

    let expected = ["child 1", "child 2", "child 3", "child 3.1", "child 3.2"];

    assert!(frame
        .descendants_breadth_first::<Tree>(root)
        .zip(expected)
        .map(|(e, expected)| {
            let name = *frame.get::<&&str>(e).unwrap();
            eprintln!("Name: {}", name);
            name == expected
        })
        .all(|val| val == true));
}

#[test]
fn builder_clone_simple() {
    let mut frame = Frame::default();
    let builder = TreeBuilderClone::<Tree>::from(("Root",));

    let root = builder.spawn(&mut frame);

    assert_eq!(*frame.get::<&&'static str>(root).unwrap(), "Root");
}

#[test]
fn builder_clone() {
    let mut frame = Frame::default();
    let mut builder = TreeBuilderClone::<Tree>::from(("root",));
    builder.attach(("child 1",));
    builder.attach({
        let mut builder = TreeBuilderClone::new();
        builder.add("child 2");
        builder
    });

    let mut tree: TreeBuilder<_> = builder.into();

    let root = tree.spawn(&mut frame);

    assert_eq!(*frame.get::<&&'static str>(root).unwrap(), "root");

    for (a, b) in frame
        .descendants_depth_first::<Tree>(root)
        .zip(["child 1", "child 2"])
    {
        assert_eq!(*frame.get::<&&str>(a).unwrap(), b)
    }
}

#[test]
fn reserve() {
    let mut frame = Frame::default();
    let mut builder = TreeBuilderClone::<Tree>::from(("root",));
    builder.attach(("child 1",));
    builder.attach({
        let mut builder = TreeBuilderClone::new();
        builder.add("child 2");
        builder
    });

    let mut tree: TreeBuilder<_> = builder.into();

    let root = tree.reserve(&frame);

    tree.spawn(&mut frame);

    assert_eq!(*frame.get::<&&'static str>(root).unwrap(), "root");

    for (a, b) in frame
        .descendants_depth_first::<Tree>(root)
        .zip(["child 1", "child 2"])
    {
        assert_eq!(*frame.get::<&&str>(a).unwrap(), b)
    }
}
