pub struct World; // 先占位

pub struct EntityID;

pub struct Texture;

pub struct KeyFrames;

#[derive(Debug, Clone, Copy)]
pub struct EID(u64);

#[derive(Debug, Clone, Copy)]
pub struct MID(u64);

#[derive(Debug, Clone, Copy)]
pub struct GID(u64);

#[derive(Debug, Clone, Copy)]
pub struct TID(u64);

#[derive(Debug, Clone, Copy)]
pub struct AID(u64);

#[derive(Debug, Clone, Copy)]
pub struct ATID(u64);

#[derive(Debug, Clone)]
pub struct UUID(String);

#[derive(Debug, Clone, Copy)]
pub struct AGID(u64);

pub trait TFactory {
    /// 创建 节点 - Node
    /// * `scaling` - [f32, f32, f32] - scale
    /// * `rotation` - [f32, f32, f32] - rotation (Euler Angle)
    /// * `rotation_quaterion` - [f32, f32, f32, f32] - rotation (Quaterion)
    /// * `matrix` - [f32; 16] - matrix
    fn create_node(
        &mut self,
        translation: Option<[f32; 3]>,
        scaling: Option<[f32; 3]>,
        rotation: Option<[f32; 3]>,
        rotation_quaterion: Option<[f32; 4]>,
        matrix: Option<[[f32; 4]; 4]>,
    ) -> EID;

    /// 赋予节点 层级信息 - node.
    /// TODO
    fn layer_mask(&self, entity: EID, layer: u32);

    /// 赋予节点 包围盒信息 -
    /// * `center` `extend` - boundingbox
    fn bounding_info(&self, entity: EID, min: [f32; 3], max: [f32; 3]);

    /// 查询是否已有 目标网格信息
    /// * `id` - mesh > geometry > primitives 数据路径
    fn query_geometry(&self, id: UUID) -> Option<GID>;

    /// 创建 网格
    /// * `positions` - 顶点坐标数据 - primitives>attributes>POSITION
    /// * `indices` - 顶点索引数据 - primitives>indices
    /// * `normals` - 顶点法线数据 - primitives>attributes>NORMAL
    /// * `tangents` - 顶点切线数据 - primitives>attributes>TANGENT
    /// * `colors` - 顶点颜色数据 - primitives>attributes>COLOR
    /// * `uvs` - 顶点uv数据 - primitives>attributes>TEXCOORD_0
    /// * `uv2s` - 顶点uv2数据 - primitives>attributes>TEXCOORD_1
    fn create_geometry_base(
        &self,
        id: UUID,
        positions: Option<Vec<[f32; 3]>>,
        indices: Option<Vec<u32>>,
        normals: Option<Vec<[f32; 3]>>,
        tangents: Option<Vec<[f32; 4]>>,
        colors: Option<Vec<[f32; 4]>>,
        uvs: Option<Vec<[f32; 2]>>,
        uv2s: Option<Vec<[f32; 2]>>,
    ) -> GID;

    /// 查询是否已有 目标材质
    /// * `id` - PI_material > instancedID
    fn query_material(&self, id: UUID) -> Option<MID>;

    /// 创建材质
    fn create_material_base(
        &mut self,
        id: UUID,
        alpha: Option<f32>,
        render_queue: Option<u32>,
        cull_face: Option<u8>,
        z_write: Option<bool>,
    ) -> MID;

    /// 查询是否已有 纹理
    fn query_texture(id: UUID) -> Option<TID>;

    /// 创建 纹理
    fn create_texture(&mut self, id: UUID, texture: Texture) -> TID;

    /// 创建 纹理 view
    /// * `` - samplers
    fn texture_view(
        &mut self,
        id: TID,
        has_alpha: Option<bool>,
        mag_filter: Option<u8>,
        min_filter: Option<u8>,
        wrap_u: Option<u8>,
        wrap_v: Option<u8>,
        format: u8,
    );

    /// 绑定 geometry - 可以多次
    fn mesh_geometry(&mut self, entity: EID, geometry: GID);

    /// 绑定 材质 - 可以多次
    fn mesh_material(&mut self, entity: EID, material: MID);

    /// 查询是否已有目标 动画数据
    fn query_animation(id: UUID) -> Option<AID>;

    /// 创建 动画数据
    /// * `keys` - 关键帧数据 buffer
    ///   * PiChannel 数组中每个元素 描述了 一个 animation 的 关键帧数据 类型, 起始&结束帧, 关键帧数据在 bufffer 中存储
    /// * `ty` - PiChannel - TODO
    fn animation(id: UUID, keys: KeyFrames, ty: u32) -> AID;

    /// 创建 Target 动画
    /// * `attr` - PiChannel - TODO
    fn target_animation(target: EID, attr: u32, ty: u32, animation: &[AID]) -> ATID;

    /// 创建 动画组
    /// * `group_name` - animations > name
    fn animation_group(target_animations: &[ATID], group_name: &str) -> AGID;
}

use std::{path::PathBuf, str::FromStr};

use gltf::{Document, Gltf};
use pi_gltf as gltf;
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

    pub async fn init(&self, factory: &mut impl TFactory) {
        let gltf = &self.gltf;
        let buffer_data = self.load_buffer().await;
        let mut materials = vec![];
        for material in self.gltf.materials() {
            materials.push(material);
        }
        for node in gltf.nodes() {
            let id = match node.transform() {
                gltf::scene::Transform::Matrix { matrix } => {
                    factory.create_node(None, None, None, None, Some(matrix))
                }
                gltf::scene::Transform::Decomposed {
                    translation,
                    rotation,
                    scale,
                } => {
                    factory.create_node(Some(translation), Some(scale), None, Some(rotation), None)
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
