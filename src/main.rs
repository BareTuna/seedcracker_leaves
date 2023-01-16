use std::{
    ops::Range,
    thread,
    time::SystemTime,
};

use seedcracker_leaves::{ChunkRandom, JavaRandom, Xoro};

fn main() {
    let available_threads = thread::available_parallelism().unwrap().get();
    let mut thread_pool = Vec::with_capacity(available_threads);

    let half_threads = available_threads as i32 / 2;
    let part = ((i32::MAX as i64 + 1) / half_threads as i64) as i32;
    for i in -half_threads..half_threads {
        let start = i.saturating_mul(part);
        let end = (i + 1).saturating_mul(part);

        println!("Kernel: {}", i + half_threads);
        println!("Seeds from {} to {}", start, end);
        println!();

        thread_pool.push(thread::spawn(move || check_seeds(start..end)));
    }

    // Wait for threads to finish
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
    let thread_that_reports_progress = range.end == i32::MAX;

    let mut random = Xoro::new(1);

    'seed_loop: for seed in range.clone() {
        let seed = seed as i64;

        if seed % 10000000 == 0 && thread_that_reports_progress {
            println!(
                "{}% at second {}",
                100.0 * (seed - range.start as i64) as f32 / (range.end - range.start) as f32,
                SystemTime::now().duration_since(time).unwrap().as_secs()
            );
        }

        let mut popseed = random.set_population_seed(
            seed,
            most_neg_coords_of_chunk.0,
            most_neg_coords_of_chunk.1,
        );
        popseed += (9 * 10000) + 20;
        random.set_seed(popseed);
        let seed = seed as i32;

        let mut tree_indices: Vec<_> = (0..trees.len()).collect();

        let mut x = random.next_i32_bounded(16);

        // Placement attempts, loose bound
        for _ in 0..100 {
            let mut z = random.next_i32_bounded(16);

            'tree_check_loop: for tree_num in &tree_indices {
                let tree = trees[*tree_num];

                // 1) Check randomly chosen (x, z) position is a tree position
                // 2) Ensure random tree trunk height matches
                // 3) Ensure the random corner leaves match

                if x != tree[0] || z != tree[1] {
                    continue;
                }

                let mut peek_random = random.clone();
                peek_random.skip(2);

                // `next_i32(3)` corresponds to the 0-2 extra logs for oak
                if peek_random.next_i32_bounded(3) != tree[2] {
                    continue;
                }

                // Skip 1 call for second additive trunk height,
                // and skip 4 calls for the upper leaves that never spawn
                peek_random.skip(5);

                for leaf_data in &tree[3..15] {
                    if peek_random.next_i32_bounded(2) != *leaf_data && *leaf_data != -1 {
                        break 'tree_check_loop;
                    }
                }

                // All checks pass for this tree!
                // 1) transfer state from `peek_random` to `random`
                // 2) ???
                // 3) Remove this tree from the check

                random = peek_random;

                z = random.next_i32_bounded(16);

                tree_indices.remove(tree_indices.iter().position(|x| *x == *tree_num).unwrap());

                // If all trees in the list match, then this is a potential seed!
                if tree_indices.len() <= 1 {
                    println!("----------------");
                    println!("Seed: {}", seed);
                    println!("----------------");
                    continue 'seed_loop;
                }
                break;
            }
            x = z;
        }
    }
}
