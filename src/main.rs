use std::{
    ops::Range,
    thread::{self, available_parallelism},
    time::SystemTime,
    rc::Rc,
    cell::RefCell,
};

use seedcracker_leaves::{
    xoro::Xoro,
    chunk_random::ChunkRandom,
};

fn main() {
    // let mut rng = JavaRandom::with_seed(0);
    // println!("{}", rng.next_i64());
    // println!("{}", rng.next_i64());
    // println!("{}", rng.next_i64());

    // submittable(0..268435455);
    // process::exit(0);


    let available_threads = available_parallelism().unwrap().get();
    let mut thread_pool = Vec::with_capacity(available_threads);

    let half_threads = available_threads as i32 / 2;
    let part = i32::MAX / half_threads;
    for i in -half_threads..half_threads {
        let start = i * part;
        let end = (i + 1) * part;

        println!("Kernel: {}", i + half_threads);
        println!("Seeds from {} to {}", start, end);
        println!();

        thread_pool.push(
            thread::spawn(move || { check_seeds(start..end) })
        );
    }

    // wait for threads to finish
    for t in thread_pool {
        t.join().unwrap();
    }
}

macro_rules! tree_token_translator {
    (?) => { -1 };
    (_) => { 0 };
    (#) => { 1 };
    ( $num:expr ) => { $num };
}

macro_rules! tree_row {
    ( [ $( $index:tt )* ] ) => {[
        $( tree_token_translator!($index) ),*
    ]};
}

macro_rules! trees {
    [ $($index:tt)* ] => {[
        $( tree_row!($index) ),*
    ]};
}

fn check_seeds(range: Range<i32>) {
    let time = SystemTime::now();

    // Leaf layout: https://cdn.discordapp.com/attachments/766084065795244092/953750827544223794/unknown.png
    // Wool is orientated like the F3 crosshair
    // Layout:
    // [chunkX, chunkZ, trunkHeight, leaves(# = leaf, _ = air, ? = unknown)]
    // `#` represents 1, `_` represents 0, `?` represents -1
    let trees = trees![
    //  [XX YY   H   L L L L   L L L L   L L L L]
        [ 3  2   1   _ ? _ ?   # ? # _   _ ? # _]
        [ 0  5   2   # ? # _   _ ? ? ?   _ ? ? ?]
        [ 4  8   1   _ ? _ ?   _ ? # ?   ? ? # #]
        [12  4   2   # # _ ?   _ ? _ ?   # ? # ?]
        [10  7   0   # ? ? ?   _ _ # ?   # # _ ?]
        [ 9 13   1   _ ? # ?   _ ? _ ?   _ ? # ?]
    ];
    let most_neg_coords_of_chunk = (64, -96);

    // Logic start:
    let default_list: Vec<_> = (0..trees.len()).collect();

    let xoro_random = Rc::new(RefCell::new(Xoro::new(1)));
    let mut random = ChunkRandom::new(Rc::clone(&xoro_random));
    let leaf_xoro_random = Rc::new(RefCell::new(Xoro::new(1)));
    let mut leaf_random = ChunkRandom::new(Rc::clone(&leaf_xoro_random));

    // have just one thread keep track of progress
    let kernel_starting_at_0 = range.start == 0;

    'seed_loop: for i32_seed in range.clone() {
        let seed = i32_seed as i64;

        if seed % 10000000 == 0 && kernel_starting_at_0 {
            println!(
                "{}% at second {}",
                (seed * 100) as f32 / range.end as f32,
                SystemTime::now().duration_since(time).unwrap().as_secs()
            );
        }

        // The coords of the most negative corner of our chunk
        let mut popseed = random.set_population_seed(
            seed,
            most_neg_coords_of_chunk.0,
            most_neg_coords_of_chunk.1
        );
        popseed += (9 * 10000) + 20;
        random.set_seed(popseed);

        let mut copy_tree_positions = default_list.clone();
        // eprintln!("copyTreePositions {:?}", copy_tree_positions);

        let mut x = random.next_int(16);

        // let mut poo = 0;
        // Placement attempts, loose bound
        for _ in 0..100 {
            let mut z = random.next_int(16);

            'tree_check_loop: for tree_num in &copy_tree_positions {
                // eprintln!("tree_num: {tree_num}");
                let tree = trees[*tree_num];
                if x == tree[0] && z == tree[1] {
                    xoro_random.borrow().copy_seed_to(&mut *leaf_xoro_random.borrow_mut());
                    leaf_xoro_random.borrow_mut().skip(2);
                    if leaf_random.next_int(3) == tree[2] {
                        // 1 burned call for second height & 4 for the
                        // upper leaves that never spawn
                        leaf_xoro_random.borrow_mut().skip(5);
                        for leaf in 3..15 {
                            let leaf_data = tree[leaf];
                            if leaf_random.next_int(2) != leaf_data
                                && leaf_data != -1
                            {
                                break 'tree_check_loop;
                            }
                        }
                        leaf_xoro_random.borrow().copy_seed_to(&mut *xoro_random.borrow_mut());
                        z = random.next_int(16);
                        // eprintln!("before remove: copyTreePositions {:?}", copy_tree_positions);
                        copy_tree_positions.remove(copy_tree_positions.iter().position(|x| *x == *tree_num).unwrap());
                        // eprintln!("after remove: copyTreePositions {:?}", copy_tree_positions);
                        if copy_tree_positions.len() < 2 {
                            println!("----------------");
                            println!("Seed: {}", i32_seed);
                            println!("----------------");
                            continue 'seed_loop;
                        }
                        break;
                    }
                }
                // if matches!(poo, 0..=1 | 90..=94 | 138|139) {
                //     eprintln!("{:?}", xoro_random.borrow().seed_range);
                //     eprintln!("{:?}", leaf_xoro_random.borrow().seed_range);
                // }
                // eprintln!("counter: {poo}");
                // poo += 1;
            }
            x = z;
        }
    }
}

