pub struct World; // 先占位

pub struct EntityID;

pub struct UUID;

pub struct Texture;

pub struct KeyFrames;

pub trait TFactory<EID, GID, MID, TID, AID, ATID, AGID> {
    /// 创建 节点 - Node
    /// * `scaling` - [f32, f32, f32] - scale
    /// * `rotation` - [f32, f32, f32] - rotation (Euler Angle)
    /// * `rotation_quaterion` - [f32, f32, f32, f32] - rotation (Quaterion)
    /// * `matrix` - [f32; 16] - matrix
    fn create_node(
        &mut self,
        translation: Option<&[f32]>,
        scaling: Option<&[f32]>,
        rotation: Option<&[f32]>,
        rotation_quaterion: Option<&[f32]>,
        matrix: Option<&[f32]>,
    ) -> EID;

    /// 赋予节点 层级信息 - node.
    /// TODO
    fn layer_mask(&mut self, entity: EID, layer: u32);

    /// 赋予节点 包围盒信息 - 
    /// * `center` `extend` - boundingbox
    fn bounding_info(&mut self, entity: EID, center: &[f32], extend: &[f32]);

    /// 查询是否已有 目标网格信息
    /// * `id` - mesh > geometry > primitives 数据路径
    fn query_geometry(id: UUID) -> Option<GID>;

    /// 创建 网格
    /// * `positions` - 顶点坐标数据 - primitives>attributes>POSITION
    /// * `indices` - 顶点索引数据 - primitives>indices
    /// * `normals` - 顶点法线数据 - primitives>attributes>NORMAL
    /// * `tangents` - 顶点切线数据 - primitives>attributes>TANGENT
    /// * `colors` - 顶点颜色数据 - primitives>attributes>COLOR
    /// * `uvs` - 顶点uv数据 - primitives>attributes>TEXCOORD_0
    /// * `uv2s` - 顶点uv2数据 - primitives>attributes>TEXCOORD_1
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
    /// * `id` - PI_material > instancedID
    fn query_material(id: UUID) -> Option<MID>;

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
