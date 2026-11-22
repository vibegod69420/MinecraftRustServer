/// Minimal self-contained stub for `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }
}

/// Faithful port of `net.minecraft.core.ClientAsset`.
pub trait ClientAsset {
    // JAVA: net.minecraft.core.ClientAsset#id
    fn id(&self) -> &Identifier;
}

/// Faithful port of `net.minecraft.core.ClientAsset$Texture`.
pub trait Texture: ClientAsset {
    // JAVA: net.minecraft.core.ClientAsset$Texture#texturePath
    fn texture_path(&self) -> &Identifier;
}

/// Faithful port of `net.minecraft.core.ClientAsset$DownloadedTexture`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadedTexture {
    pub texture_path: Identifier,
    pub url: String,
}

impl DownloadedTexture {
    // JAVA: net.minecraft.core.ClientAsset$DownloadedTexture#<init>
    pub fn new(texture_path: Identifier, url: impl Into<String>) -> Self {
        Self {
            texture_path,
            url: url.into(),
        }
    }

    // JAVA: net.minecraft.core.ClientAsset$DownloadedTexture#texturePath
    pub fn texture_path(&self) -> &Identifier {
        &self.texture_path
    }

    // JAVA: net.minecraft.core.ClientAsset$DownloadedTexture#url
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl ClientAsset for DownloadedTexture {
    // JAVA: net.minecraft.core.ClientAsset$DownloadedTexture#id
    fn id(&self) -> &Identifier {
        &self.texture_path
    }
}

impl Texture for DownloadedTexture {
    // JAVA: net.minecraft.core.ClientAsset$DownloadedTexture#texturePath
    fn texture_path(&self) -> &Identifier {
        &self.texture_path
    }
}

/// Faithful port of `net.minecraft.core.ClientAsset$ResourceTexture`.
///
/// Codec/StreamCodec fields from Java are omitted for self-containment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceTexture {
    pub id: Identifier,
    pub texture_path: Identifier,
}

impl ResourceTexture {
    // JAVA: net.minecraft.core.ClientAsset$ResourceTexture#<init>
    pub fn new(id: Identifier) -> Self {
        let texture_path = Identifier {
            namespace: id.namespace.clone(),
            path: format!("textures/{}.png", id.path),
        };
        Self { id, texture_path }
    }

    // JAVA: net.minecraft.core.ClientAsset$ResourceTexture#<init>
    pub fn from_parts(id: Identifier, texture_path: Identifier) -> Self {
        Self { id, texture_path }
    }

    // JAVA: net.minecraft.core.ClientAsset$ResourceTexture#id
    pub fn id(&self) -> &Identifier {
        &self.id
    }

    // JAVA: net.minecraft.core.ClientAsset$ResourceTexture#texturePath
    pub fn texture_path(&self) -> &Identifier {
        &self.texture_path
    }
}

impl ClientAsset for ResourceTexture {
    // JAVA: net.minecraft.core.ClientAsset$ResourceTexture#id
    fn id(&self) -> &Identifier {
        &self.id
    }
}

impl Texture for ResourceTexture {
    // JAVA: net.minecraft.core.ClientAsset$ResourceTexture#texturePath
    fn texture_path(&self) -> &Identifier {
        &self.texture_path
    }
}

#[cfg(test)]
mod tests {
    use super::{ClientAsset, DownloadedTexture, Identifier, ResourceTexture, Texture};

    #[test]
    fn resource_texture_single_arg_constructor_builds_texture_path() {
        let id = Identifier::new("minecraft", "entity/player/wide/steve");
        let tex = ResourceTexture::new(id.clone());
        assert_eq!(tex.id(), &id);
        assert_eq!(
            tex.texture_path(),
            &Identifier::new("minecraft", "textures/entity/player/wide/steve.png")
        );
    }

    #[test]
    fn resource_texture_from_parts_preserves_both() {
        let id = Identifier::new("minecraft", "icon");
        let path = Identifier::new("minecraft", "textures/custom/icon.png");
        let tex = ResourceTexture::from_parts(id.clone(), path.clone());
        assert_eq!(tex.id(), &id);
        assert_eq!(tex.texture_path(), &path);
    }

    #[test]
    fn downloaded_texture_id_is_texture_path() {
        let tex_path = Identifier::new("minecraft", "skins/abc");
        let dt = DownloadedTexture::new(tex_path.clone(), "https://example.com/skin.png");
        assert_eq!(dt.id(), &tex_path);
        assert_eq!(Texture::texture_path(&dt), &tex_path);
        assert_eq!(dt.url(), "https://example.com/skin.png");
    }

    #[test]
    fn trait_objects_expose_expected_ids() {
        let rt = ResourceTexture::new(Identifier::new("m", "a/b"));
        let dt = DownloadedTexture::new(Identifier::new("m", "c"), "u");
        let a: &dyn ClientAsset = &rt;
        let b: &dyn ClientAsset = &dt;
        assert_eq!(a.id().path, "a/b");
        assert_eq!(b.id().path, "c");
    }
}
