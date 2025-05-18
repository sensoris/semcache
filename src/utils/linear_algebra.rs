pub fn normalize(vec: Vec<f32>) -> Vec<f32> {
    let norm = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    vec.into_iter().map(|x| x / norm).collect()
}
