use std::{path::PathBuf, str::FromStr};

use gltf::{Document, Gltf};
use pi_gltf as gltf;

use crate::interface::InterfaceGLTFLoader;
pub struct GltfLoader {
    gltf: Gltf,
    path: PathBuf,
}

impl GltfLoader {
    pub async fn from_gltf(path: &str) -> Result<Self, String> {
        let data = pi_hal::file::from_path_or_url(path).await;
        match Gltf::from_slice_without_validation(&data) {
            Ok(gltf) => {
                return Ok(Self {
                    gltf,
                    path: PathBuf::from_str(path).unwrap(),
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

    pub async fn from_cbor(path: &str) -> Result<Self, String> {
        let data = pi_hal::file::from_path_or_url(path).await;

        match serde_cbor::from_slice(&data) {
            Ok(root) => {
                let document = Document::from_json_without_validation(root);
                let gltf = Gltf {
                    blob: Some(data),
                    document,
                };
                return Ok(Self {
                    gltf,
                    path: PathBuf::from_str(path).unwrap(),
                });
            }
            Err(err) => {
                return Err(format!(
                    "create gltf form cbor failed!! path: {}, reason: {:?}",
                    path, err
                ))
            }
        }
    }

    pub async fn init(&self, factory: &mut impl InterfaceGLTFLoader) {
        let gltf = &self.gltf;
        let buffer_data = self.load_buffer().await;
        let mut materials = vec![];
        for material in self.gltf.materials() {
            materials.push(material);
        }
        for node in gltf.nodes() {
            let id = match node.transform() {
                gltf::scene::Transform::Matrix { matrix } => {
                    factory.gltf_create_node(None, None, None, None, Some(matrix))
                }
                gltf::scene::Transform::Decomposed {
                    translation,
                    rotation,
                    scale,
                } => {
                    factory.gltf_create_node(Some(translation), Some(scale), None, Some(rotation), None)
                }
            };

            if let Some(mesh) = node.mesh() {
                // TODO: 取 layer 参数
                // factory.layer_mask(entity, layer);

                // 用name加索引作为uuid
                let name = match mesh.name() {
                    Some(str) => str.to_string(),
                    // TODO 生成独一无二字符
                    None => "".to_string(),
                };
                let uuid = UUID(name);
                for primitive in mesh.primitives() {
                    if factory.query_geometry(uuid.clone()).is_none() {
                        let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

                        let positions = reader
                            .read_positions()
                            .map(|v| v.collect::<Vec<[f32; 3]>>());

                        let indices = reader
                            .read_indices()
                            .map(|v| v.into_u32().collect::<Vec<u32>>());

                        let normals = reader.read_normals().map(|v| v.collect::<Vec<[f32; 3]>>());

                        let tangents = reader.read_tangents().map(|v| v.collect::<Vec<[f32; 4]>>());

                        let colors = reader
                            .read_colors(0)
                            .map(|v| v.into_rgba_f32().collect::<Vec<[f32; 4]>>());

                        let uvs = reader
                            .read_tex_coords(0)
                            .map(|v| v.into_f32().collect::<Vec<[f32; 2]>>());

                        let uv2s = reader
                            .read_tex_coords(1)
                            .map(|v| v.into_f32().collect::<Vec<[f32; 2]>>());

                        let gid = factory.create_geometry_base(
                            uuid.clone(),
                            positions,
                            indices,
                            normals,
                            tangents,
                            colors,
                            uvs,
                            uv2s,
                        );
                        factory.mesh_geometry(id, gid);

                        if let Some(material) = primitive
                            .material()
                            .index()
                            .and_then(|i| materials.get(i).cloned())
                        {
                            if factory.query_material(uuid.clone()).is_none(){
                                // TODO 扩展 
                                // let mid = factory.create_material_base(uuid, alpha, render_queue, cull_face, z_write);
                            }
                        }
                    }
                }
            }
        }

        for r in gltf.materials() {}
    }

    async fn load_buffer(&self) -> Vec<Vec<u8>> {
        let mut buffer_data: Vec<Vec<u8>> = Vec::new();
        for buffer in self.gltf.buffers() {
            println!("source: {:?}", buffer.source());
            match buffer.source() {
                gltf::buffer::Source::Uri(uri) => {
                    let path = self.path.join(uri).to_str().unwrap().to_string();
                    let data = pi_hal::file::from_path_or_url(&path).await;
                    buffer_data.push(data);
                }
                gltf::buffer::Source::Bin => {
                    let r = self.gltf.blob.as_deref().unwrap();
                    buffer_data.push(r.into());
                }
            }
        }

        buffer_data
    }

    pub fn load(&self) {}
}
