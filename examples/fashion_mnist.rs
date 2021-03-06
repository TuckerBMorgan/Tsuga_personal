use ndarray::prelude::*;
use tsuga::prelude::*;

use minifb::{Key, ScaleMode, Window, WindowOptions};
use mnist::*;
use ndarray_stats::QuantileExt;
use rand::prelude::*;

// Currently runs using the Fashion MNIST dataset
const LABELS: &[&'static str] = &[
    "T-shirt",
    "Trouser",
    "Pullover",
    "Dress",
    "Coat",
    "Sandal",
    "Shirt",
    "Sneaker",
    "Bag",
    "Ankle boot",
];

fn main() {
    let (input, output, test_input, test_output) = mnist_as_ndarray();
    println!("Successfully unpacked the MNIST dataset into Array2<f32> format!");

    // Let's see an example of the parsed MNIST dataset on both the training and testing data
    let mut rng = rand::thread_rng();
    let mut num: usize = rng.gen_range(0, input.nrows());
    println!(
        "Input record #{} has a label of {}",
        num,
        return_label_from_one_hot(output.slice(s![num, ..]))
    );
    display_img(input.slice(s![num, ..]).to_owned());

    num = rng.gen_range(0, test_input.nrows());
    println!(
        "Test record #{} has a label of {}",
        num,
        return_label_from_one_hot(test_output.slice(s![num, ..]))
    );
    display_img(test_input.slice(s![num, ..]).to_owned());

    // Now we can begin configuring any additional hidden layers, specifying their size and activation function
    let mut layers_cfg: Vec<FCLayer> = Vec::new();
    let sigmoid_layer_0 = FCLayer::new("sigmoid", 128);
    layers_cfg.push(sigmoid_layer_0);
    let sigmoid_layer_1 = FCLayer::new("sigmoid", 64);
    layers_cfg.push(sigmoid_layer_1);

    // The network can now be built using the specified layer configurations
    // Several other options for tuning the network's performance are available as well
    let mut fcn = FullyConnectedNetwork::default(input, output)
        .add_layers(layers_cfg)
        .iterations(10_000)
        // .min_iterations(2_500)
        .error_threshold(0.003)
        .learnrate(0.01)
        .batch_size(200)
        .validation_pct(0.005)
        .build();

    // Training occurs in place on the network
    fcn.train().expect("An error occurred during training");

    // We can now pass an appropriately-sized input through our trained network,
    // receiving an Array2<f32> on the output
    let test_result = fcn.evaluate(test_input.clone());

    // And will compare that output against the ideal one-hot encoded testing label array
    compare_results(test_result.clone(), test_output);

    // Now display a singular value with the classification spread to see an example of the actual values
    num = rng.gen_range(0, test_input.nrows());
    println!(
        "Test result #{} has a classification spread of:\n------------------------------",
        num
    );
    for i in 0..LABELS.len() {
        println!("{}: {:.2}%", LABELS[i], test_result[[num, i]] * 100.);
    }

    display_img(test_input.slice(s![num, ..]).to_owned());
}

fn mnist_as_ndarray() -> (Array2<f32>, Array2<f32>, Array2<f32>, Array2<f32>) {
    let (trn_size, _rows, _cols) = (60_000, 28, 28);
    let tst_size = 10_000;

    // Deconstruct the returned Mnist struct.
    // YOu can see the default Mnist struct at https://docs.rs/mnist/0.4.0/mnist/struct.MnistBuilder.html
    let Mnist {
        trn_img,
        trn_lbl,
        tst_img,
        tst_lbl,
        ..
    } = MnistBuilder::new()
        .base_path("data/fashion")
        .use_fashion_data()
        .download_and_extract()
        .label_format_one_hot()
        .finalize();

    // Convert the returned Mnist struct to Array2 format
    let trn_lbl: Array2<f32> = Array2::from_shape_vec((trn_size, 10), trn_lbl)
        .expect("Error converting labels to Array2 struct")
        .map(|x| *x as f32);
    // println!("The first digit is a {:?}",trn_lbl.slice(s![image_num, ..]) );

    // Can use an Array2 or Array3 here (Array3 for visualization)
    let trn_img = Array2::from_shape_vec((trn_size, 784), trn_img)
        .expect("Error converting images to Array3 struct")
        .map(|x| *x as f32 / 256.);
    // println!("{:#.0}\n",trn_img.slice(s![image_num, .., ..]));

    // Convert the returned Mnist struct to Array2 format
    let tst_lbl: Array2<f32> = Array2::from_shape_vec((tst_size, 10), tst_lbl)
        .expect("Error converting labels to Array2 struct")
        .map(|x| *x as f32);

    let tst_img = Array2::from_shape_vec((tst_size, 784), tst_img)
        .expect("Error converting images to Array3 struct")
        .map(|x| *x as f32 / 256.);

    (trn_img, trn_lbl, tst_img, tst_lbl)
}

fn compare_results(actual: Array2<f32>, ideal: Array2<f32>) {
    let mut correct_number = 0;
    for i in 0..actual.nrows() {
        let result_row = actual.slice(s![i, ..]);
        let output_row = ideal.slice(s![i, ..]);

        if result_row.argmax() == output_row.argmax() {
            correct_number += 1;
        }
    }
    println!(
        "Total correct values: {}/{}, or {}%",
        correct_number,
        actual.nrows(),
        (correct_number as f32) * 100. / (actual.nrows() as f32)
    );
}

// Displays in an MNIST image in a pop-up window
fn display_img(input: Array1<f32>) {
    let img_vec: Vec<u8> = input.to_vec().iter().map(|x| (*x * 256.) as u8).collect();
    // println!("img_vec: {:?}",img_vec);
    let mut buffer: Vec<u32> = Vec::with_capacity(28 * 28);
    for px in 0..784 {
        let temp: [u8; 4] = [img_vec[px], img_vec[px], img_vec[px], 255u8];
        // println!("temp: {:?}",temp);
        buffer.push(u32::from_le_bytes(temp));
    }

    let (window_width, window_height) = (600, 600);
    let mut window = Window::new(
        "Test - ESC to exit",
        window_width,
        window_height,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::Center,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) && !window.is_key_down(Key::Q) {
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, 28, 28).unwrap();
    }
}

fn return_label_from_one_hot(one_hot: ArrayView1<f32>) -> String {
    let one_hot = one_hot.mapv(|x| x as u8);
    if one_hot == array![1, 0, 0, 0, 0, 0, 0, 0, 0, 0] {
        "T-shirt".to_string()
    } else if one_hot == array![0, 1, 0, 0, 0, 0, 0, 0, 0, 0] {
        "Trouser".to_string()
    } else if one_hot == array![0, 0, 1, 0, 0, 0, 0, 0, 0, 0] {
        "Pullover".to_string()
    } else if one_hot == array![0, 0, 0, 1, 0, 0, 0, 0, 0, 0] {
        "Dress".to_string()
    } else if one_hot == array![0, 0, 0, 0, 1, 0, 0, 0, 0, 0] {
        "Coat".to_string()
    } else if one_hot == array![0, 0, 0, 0, 0, 1, 0, 0, 0, 0] {
        "Sandal".to_string()
    } else if one_hot == array![0, 0, 0, 0, 0, 0, 1, 0, 0, 0] {
        "Shirt".to_string()
    } else if one_hot == array![0, 0, 0, 0, 0, 0, 0, 1, 0, 0] {
        "Sneaker".to_string()
    } else if one_hot == array![0, 0, 0, 0, 0, 0, 0, 0, 1, 0] {
        "Bag".to_string()
    } else if one_hot == array![0, 0, 0, 0, 0, 0, 0, 0, 0, 1] {
        "Ankle boot".to_string()
    } else {
        format!("Error: no valid label could be assigned to {}", one_hot).to_string()
    }
}
