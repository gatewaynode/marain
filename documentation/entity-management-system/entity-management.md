# Entity management

The application treats the YAML files in the `/schema/` folder as the authoritative source of truth
for all entities in the system.  These entities are hot reloaded when files change in the directory
, are deleted, or are loaded.  The entities are parsed from the YAML and stored in a globally accessible Trait-object (Box) called `entity_definitions`.

## Technique

Trait‑object (Box<dyn MyTrait>) 
– each element is a boxed value that implements a common trait.	
- The set of element types may grow, you want true polymorphism, or the structs are large and you don’t want them copied into the enum.

Trait‑object Vec (Box<dyn Drawable>)

### Example Implementation

```rust
// ----- main.rs --------------------------------------------------------------
use std::fmt::Debug;

// The common behaviour.
trait Drawable: Debug {
    fn draw(&self);
}

// Concrete types – they can be as large or complex as you like.
#[derive(Debug)]
struct Circle {
    radius: f64,
}
impl Drawable for Circle {
    fn draw(&self) {
        println!("(dyn) Circle with r = {}", self.radius);
    }
}

#[derive(Debug)]
struct Rectangle {
    width:  f64,
    height: f64,
}
impl Drawable for Rectangle {
    fn draw(&self) {
        println!(
            "(dyn) Rectangle {} × {}",
            self.width, self.height
        );
    }
}

// A third type just to prove heterogeneity.
#[derive(Debug)]
struct Star {
    points: usize,
}
impl Drawable for Star {
    fn draw(&self) {
        println!("(dyn) Star with {} points", self.points);
    }
}

/* ------------------------------------------------------------------------ */
fn main() {
    // Vec of boxed trait objects. Each element lives on the heap.
    let mut objects: Vec<Box<dyn Drawable>> = Vec::new();

    objects.push(Box::new(Circle { radius: 1.2 }));
    objects.push(Box::new(Rectangle {
        width: 3.0,
        height: 4.5,
    }));
    objects.push(Box::new(Star { points: 5 }));

    // Dynamic dispatch – the concrete `draw` implementation is chosen at runtime.
    for obj in &objects {
        obj.draw();
    }

    // Because we required `Debug`, we can also debug‑print each element:
    println!("--- Debug view of the vector ---");
    for (i, obj) in objects.iter().enumerate() {
        println!("{}: {:?}", i, obj);
    }
}
```

#### What’s happening?
Box<dyn Drawable> is a fat pointer: it stores a pointer to the heap‑allocated concrete value plus a v‑table pointer for dynamic dispatch.
The vector holds only those fat pointers; each element can be any type that implements Drawable.

- Pros
  - Adding a new drawable type never touches existing code – just implement Drawable and push it into the vec.
  - Works even when the concrete types are defined in different crates.
- Cons
  - One heap allocation per element (or you can use Rc<dyn Drawable>/Arc<dyn Drawable> to share).
  - Slight runtime overhead for virtual‑function