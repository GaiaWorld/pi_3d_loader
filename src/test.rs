use std::{
    path::PathBuf,
    str::FromStr,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};

use gltf::{Document, Gltf};
use pi_atom::Atom;
use pi_gltf as gltf;

pub struct GltfTest {
    gltf: Gltf,
    path: PathBuf,
    buffer_data: Vec<(String, Vec<u8>)>,
}

impl GltfTest {
    pub fn position(&self) -> Option<Vec<[f32; 3]>> {
        for node in self.gltf.nodes() {
            if let Some(mesh) = node.mesh() {
                for primitive in mesh.primitives() {
                    let reader =
                        primitive.reader(|buffer| Some(&self.buffer_data[buffer.index()].1));

                    if let Some(positions) = reader
                        .read_positions()
                        .map(|v| v.collect::<Vec<[f32; 3]>>())
                    {
                        return Some(positions);
                    }
                }
            }
        }
        None
    }

    pub fn indices(&self) -> Option<Vec<u32>> {
        for node in self.gltf.nodes() {
            if let Some(mesh) = node.mesh() {
                for primitive in mesh.primitives() {
                    let reader =
                        primitive.reader(|buffer| Some(&self.buffer_data[buffer.index()].1));

                    if let Some(indices) = reader
                        .read_indices()
                        .map(|v| v.into_u32().collect::<Vec<u32>>())
                    {
                        return Some(indices);
                    }
                }
            }
        }
        None
    }

    pub fn joints(&self) -> Option<Vec<[u16; 4]>> {
        for node in self.gltf.nodes() {
            if let Some(mesh) = node.mesh() {
                for primitive in mesh.primitives() {
                    let reader =
                        primitive.reader(|buffer| Some(&self.buffer_data[buffer.index()].1));

                    if let Some(joints) = reader
                        .read_joints(0)
                        .map(|v| v.into_u16().collect::<Vec<[u16; 4]>>())
                    {
                        return Some(joints);
                    }
                }
            }
        }
        None
    }

    pub fn weights(&self) -> Option<Vec<[f32; 4]>> {
        for node in self.gltf.nodes() {
            if let Some(mesh) = node.mesh() {
                for primitive in mesh.primitives() {
                    let reader =
                        primitive.reader(|buffer| Some(&self.buffer_data[buffer.index()].1));

                    if let Some(weights) = reader
                        .read_weights(0)
                        .map(|v| v.into_f32().collect::<Vec<[f32; 4]>>())
                    {
                        return Some(weights);
                    }
                }
            }
        }
        None
    }

    pub fn bones(&self) -> Option<Vec<[[f32; 4]; 4]>> {
        for node in self.gltf.nodes() {
            if let Some(skin) = node.skin() {
                let reader = skin.reader(|buffer| Some(&self.buffer_data[buffer.index()].1));

                if let Some(bones) = reader.read_inverse_bind_matrices() {
                    let bones = bones.collect::<Vec<[[f32; 4]; 4]>>();
                    return Some(bones);
                }
            }
        }

        None
    }
}

pub fn from_gltf(path: &str) -> Result<GltfTest, String> {
    let data = std::fs::read(path).unwrap();
    match Gltf::from_slice_without_validation(&data) {
        Ok(gltf) => {
            let path = PathBuf::from_str(path).unwrap();

            let mut buffer_data: Vec<(String, Vec<u8>)> = Vec::new();
            for buffer in gltf.buffers() {
                // println!("source: {:?}", buffer.source());
                match buffer.source() {
                    gltf::buffer::Source::Uri(uri) => {
                        // println!("========= uri: {}", uri);
                        if uri.starts_with("data:") {
                            if let Some(index) = uri.find(',') {
                                let base64_buffer = uri.split_at(index + 1).1;
                                println!("base64_buffer: {}", base64_buffer);
                                let buffer = base64::decode(base64_buffer).unwrap();
                                buffer_data.push(("".to_string(), buffer));
                            }
                        }
                    }
                    gltf::buffer::Source::Bin => {
                        let r = gltf.blob.as_deref().unwrap();
                        // TODO: 不會用base64 數據
                        buffer_data.push(("".to_string(), r.into()));
                    }
                }
            }
            return Ok(GltfTest {
                gltf,
                path,
                buffer_data,
            });
        }

        Err(err) => {
            return Err(format!(
                "create gltf  failed!! path: {}, reason: {:?}",
                path, err
            ))
        }
    };
}

#[test]
fn test_gltf() {
    let gltf_test = from_gltf("E:/rust_render/pi_3d_loader/SimpleSkin.gltf").unwrap();
    println!("gltf_test.bones(): {:?}", gltf_test.bones());

    println!("gltf_test.position(): {:?}", gltf_test.position());
    println!("gltf_test.indices(): {:?}", gltf_test.indices());
    println!("gltf_test.joints(): {:?}", gltf_test.joints());
    println!("gltf_test.weights(): {:?}", gltf_test.weights());
}
