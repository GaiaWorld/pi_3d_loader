pub struct World; // 先占位

pub struct EntityID;

pub struct UUID;

pub struct Texture;

pub struct KeyFrames;

pub trait TFactory<EID, GID, MID, TID, AID, ATID, AGID> {
    /// 创建 节点
    fn create_node(
        &mut self,
        position: Option<&[f32]>,
        scaling: Option<&[f32]>,
        rotation: Option<&[f32]>,
        rotation_quaterion: Option<&[f32]>,
    ) -> EID;

    /// 赋予节点 层级信息
    fn layer_mask(&mut self, entity: EID, layer: u32);

    /// 赋予节点 包围盒信息
    fn bounding_info(&mut self, entity: EID, center: &[f32], extend: &[f32]);

    /// 查询是否已有 目标网格信息
    fn query_geometry(id: UUID) -> Option<GID>;

    /// 创建 网格
    fn create_geometry_base(
        &mut self,
        id: UUID,
        positions: &[f32],
        indices: &[u16],
        normals: Option<&[f32]>,
        tangents: Option<&[f32]>,
        colors: Option<&[f32]>,
        uvs: Option<&[f32]>,
        uv2s: Option<&[f32]>,
    ) -> GID;

    /// 查询是否已有 目标材质
    fn query_material(id: UUID) -> Option<MID>;

    /// 创建材质
    fn create_material_base(
        &mut self,
        id: UUID,
        alpha: Option<f32>,
        alpha_index: Option<u32>,
        cull_face: Option<u8>,
        z_write: Option<bool>,
    ) -> MID;

    /// 查询是否已有 纹理
    fn query_texture(id: UUID) -> Option<TID>;

    /// 创建 纹理
    fn create_texture(&mut self, id: UUID, texture: Texture) -> TID;

    /// 创建 纹理 view
    fn texture_view(
        &mut self,
        id: TID,
        has_alpha: bool,
        mag_filter: u8,
        min_filter: u8,
        wrap_u: u8,
        wrap_v: u8,
        format: u8,
    );

    /// 绑定 geometry - 可以多次
    fn mesh_geometry(&mut self, entity: EID, geometry: GID);

    /// 绑定 材质 - 可以多次
    fn mesh_material(&mut self, entity: EID, material: MID);

    /// 查询是否已有目标 动画数据
    fn query_animation(id: UUID) -> Option<AID>;

    /// 创建 动画数据
    fn animation(id: UUID, keys: KeyFrames, ty: u32) -> AID;

    /// 创建 Target 动画
    fn target_animation(target: EID, attr: u32, ty: u32, animation: &[AID]) -> ATID;

    /// 创建 动画组
    fn animation_group(target_animations: &[ATID]) -> AGID;
}

use gltf::{Document, Gltf};
use pi_gltf as gltf;
pub struct GltfLoader(Gltf);

impl GltfLoader {
    pub async fn from_gltf(path: &str) -> Result<Self, String> {
        let data = pi_hal::file::from_path_or_url(path).await;
        match Gltf::from_slice_without_validation(&data) {
            Ok(gltf) => {
                return Ok(Self(gltf));
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
                return Ok(Self(gltf));
            }
            Err(err) => {
                return Err(format!(
                    "create gltf form cbor failed!! path: {}, reason: {:?}",
                    path, err
                ))
            }
        }
    }

    pub fn init<EID, GID, MID, TID, AID, ATID, AGID>(
        &self,
        factory: &mut impl TFactory<EID, GID, MID, TID, AID, ATID, AGID>,
    ) {
        let gltf = &self.0;

        for node in gltf.nodes() {
           let id = factory.create_node(None, None, None, None)
        }
    }
}
