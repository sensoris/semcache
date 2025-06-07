pub fn normalize(vec: &[f32]) -> Vec<f32> {
    let norm = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    vec.iter().map(|x| x / norm).collect()
}
