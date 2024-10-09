use std::error::Error;

use moss_hecs::{Entity, Frame};
use moss_hecs_hierarchy::*;

fn main() -> Result<(), Box<dyn Error>> {
    // Marker type which allows several hierarchies.
    struct Tree;

    let mut frame = moss_hecs::Frame::default();

    // Create a root entity, there can be several.
    let root = frame.spawn(("Root",));

    // Create a loose entity
    let child = frame.spawn(("Child 1",));

    // Attaches the child to a parent, in this case `root`
    frame.attach::<Tree>(child, root).unwrap();

    // Iterate children
    for child in frame.children::<Tree>(root) {
        let name = frame.get::<&&str>(child).unwrap();
        println!("Child: {:?} {}", child, *name);
    }

    // Add a grandchild
    frame.attach_new::<Tree, _>(child, ("Grandchild",)).unwrap();

    // Iterate recursively
    for child in frame.descendants_depth_first::<Tree>(root) {
        let name = frame.get::<&&str>(child).unwrap();
        println!("Child: {:?} {}", child, *name)
    }

    // Detach `child` and `grandchild`
    frame.detach::<Tree>(child).unwrap();

    let child2 = frame.attach_new::<Tree, _>(root, ("Child 2",)).unwrap();

    // Reattach as a child of `child2`
    frame.attach::<Tree>(child, child2).unwrap();

    frame.attach_new::<Tree, _>(root, ("Child 3",)).unwrap();

    // Hierarchy now looks like this:
    // Root
    // |-------- Child 3
    // |-------- Child 2
    //           |-------- Child 1
    //                     |-------- Grandchild

    print_tree::<Tree>(&frame, root);

    frame.despawn_all::<Tree>(child2);

    print_tree::<Tree>(&frame, root);

    frame
        .iter()
        .for_each(|entity| println!("Entity: {:?}", entity.entity()));

    Ok(())
}

fn print_tree<T: 'static + Send + Sync>(frame: &Frame, root: Entity) {
    fn internal<T: 'static + Send + Sync>(frame: &Frame, parent: Entity, depth: usize) {
        for child in frame.children::<T>(parent) {
            let name = frame.get::<&&str>(child).unwrap();
            println!(
                "{}|-------- {}",
                std::iter::repeat(" ")
                    .take((depth - 1) * 10)
                    .collect::<String>(),
                *name,
            );

            internal::<T>(frame, child, depth + 1)
        }
    }

    let name = frame.get::<&&str>(root).unwrap();
    println!("{}", *name);
    internal::<T>(frame, root, 1)
}
