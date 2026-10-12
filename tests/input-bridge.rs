// Compile this harness with target_os="ios" to exercise production input.rs
// against its atomic memory ABI. This is not a device/gameplay test.
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::OnceLock;

#[derive(Debug)]
struct TestMemory { words: UnsafeCell<[u64; 128]> }
// Shared accesses in this harness use the same aligned atomic words as input.rs.
unsafe impl Sync for TestMemory {}
#[derive(Debug)]
struct Runtime { mem: TestMemory }
static RT: OnceLock<Runtime> = OnceLock::new();

fn mem_base_len(mem: &TestMemory) -> (*mut u8, usize) {
    (mem.words.get().cast::<u8>(), 1024)
}

#[path = "../src/input.rs"]
mod input;

#[cfg(target_os = "ios")]
#[test]
fn ios_menu_input_preserves_position_deltas_lock_and_release_for_engine_polling() {
    const BLOCK: usize = 64;
    RT.set(Runtime { mem: TestMemory { words: UnsafeCell::new([0; 128]) } }).unwrap();
    let word = |index: usize| -> &AtomicI32 {
        let (base, _) = mem_base_len(&RT.get().unwrap().mem);
        unsafe { &*base.add(BLOCK + 256 + 4 * index).cast::<AtomicI32>() }
    };
    let read = |index: usize| word(index).load(Ordering::SeqCst);
    input::set_size(1280, 720);
    input::publish(BLOCK as u64);
    assert_eq!((read(0), read(1), read(7)), (-9999, -9999, 1));

    input::set_menu_touch(true);
    assert_eq!((read(0), read(1), read(2), read(3), read(7)), (640, 360, 0, 0, 1));
    // Requests from the window/controller must not unlock the game input.
    input::set_locked(false);
    assert_eq!(read(7), 1);
    input::mouse_pos(720, 410);
    input::button(0, true);
    assert_eq!((read(0), read(1), read(2), read(3), read(5)), (720, 410, 80, 50, 1));

    // Simulate consumption on one game frame, then the next drag event.
    word(2).store(0, Ordering::SeqCst);
    word(3).store(0, Ordering::SeqCst);
    input::mouse_pos(744, 400);
    assert_eq!((read(2), read(3)), (24, -10));

    // Pinch stops the drag but retains its pivot so wheel events hit the map.
    input::button(0, false);
    input::mouse_pos(800, 440);
    input::wheel(2);
    assert_eq!((read(0), read(1), read(4), read(5), read(7)), (800, 440, 2, 0, 1));
    input::set_locked(true);
    assert_eq!((read(0), read(1)), (800, 440));
    // Button-up and the last location remain visible on a later game frame.
    assert_eq!(read(5), 0);

    input::set_menu_touch(false);
    assert_eq!((read(0), read(1), read(2), read(3), read(4), read(5), read(7)), (-9999, -9999, 0, 0, 0, 0, 1));
    input::set_menu_touch(true);
    input::mouse_pos(641, 362);
    assert_eq!((read(2), read(3)), (1, 2));
}
