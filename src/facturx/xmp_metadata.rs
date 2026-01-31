//! Gestion des métadonnées XMP pour la conformité PDF/A-3 Factur-X
//!
//! Ce module fournit :
//! - La génération des métadonnées XMP conformes au standard Factur-X
//! - La validation des métadonnées avant création du PDF

use chrono::Utc;

/// Profil Factur-X utilisé
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum FacturXProfile {
    Minimum,
    BasicWL,
    Basic,
    EN16931,
    Extended,
}

impl FacturXProfile {
    /// Retourne l'identifiant URN du profil
    #[allow(dead_code)]
    pub fn urn(&self) -> &'static str {
        match self {
            FacturXProfile::Minimum => "urn:factur-x.eu:1p0:minimum",
            FacturXProfile::BasicWL => "urn:factur-x.eu:1p0:basicwl",
            FacturXProfile::Basic => "urn:factur-x.eu:1p0:basic",
            FacturXProfile::EN16931 => "urn:factur-x.eu:1p0:en16931",
            FacturXProfile::Extended => "urn:factur-x.eu:1p0:extended",
        }
    }

    /// Retourne le nom du profil pour les métadonnées XMP
    pub fn name(&self) -> &'static str {
        match self {
            FacturXProfile::Minimum => "MINIMUM",
            FacturXProfile::BasicWL => "BASIC WL",
            FacturXProfile::Basic => "BASIC",
            FacturXProfile::EN16931 => "EN 16931",
            FacturXProfile::Extended => "EXTENDED",
        }
    }
}

/// Structure contenant les informations nécessaires pour les métadonnées XMP
#[derive(Debug, Clone)]
pub struct XmpMetadata {
    /// Titre du document (ex: "Facture FA-2024-001")
    pub title: String,
    /// Nom de l'auteur/créateur (nom de l'émetteur)
    pub author: String,
    /// Sujet du document
    pub subject: String,
    /// Profil Factur-X utilisé
    pub profile: FacturXProfile,
    /// Nom du fichier XML embarqué
    pub xml_filename: String,
    /// Version Factur-X
    pub facturx_version: String,
}

impl Default for XmpMetadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            subject: "Facture électronique Factur-X".to_string(),
            profile: FacturXProfile::Minimum,
            xml_filename: "factur-x.xml".to_string(),
            facturx_version: "1.0".to_string(),
        }
    }
}

/// Erreurs de validation des métadonnées XMP
#[derive(Debug, Clone)]
pub struct XmpValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for XmpValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Résultat de la validation des métadonnées XMP
#[derive(Debug)]
pub struct XmpValidationResult {
    pub is_valid: bool,
    pub errors: Vec<XmpValidationError>,
    pub warnings: Vec<String>,
}

impl XmpValidationResult {
    #[allow(dead_code)]
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_error(field: &str, message: &str) -> Self {
        Self {
            is_valid: false,
            errors: vec![XmpValidationError {
                field: field.to_string(),
                message: message.to_string(),
            }],
            warnings: Vec::new(),
        }
    }
}

/// Valide les métadonnées XMP avant génération du PDF
///
/// Vérifie que toutes les informations requises pour la conformité
/// PDF/A-3 et Factur-X sont présentes et valides.
pub fn validate_xmp_metadata(metadata: &XmpMetadata) -> XmpValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Validation du titre
    if metadata.title.is_empty() {
        errors.push(XmpValidationError {
            field: "title".to_string(),
            message: "Le titre du document est requis pour PDF/A-3".to_string(),
        });
    }

    // Validation de l'auteur
    if metadata.author.is_empty() {
        errors.push(XmpValidationError {
            field: "author".to_string(),
            message: "L'auteur du document est requis pour PDF/A-3".to_string(),
        });
    }

    // Validation du nom de fichier XML
    if metadata.xml_filename.is_empty() {
        errors.push(XmpValidationError {
            field: "xml_filename".to_string(),
            message: "Le nom du fichier XML embarqué est requis".to_string(),
        });
    } else if !metadata.xml_filename.ends_with(".xml") {
        errors.push(XmpValidationError {
            field: "xml_filename".to_string(),
            message: "Le fichier XML doit avoir l'extension .xml".to_string(),
        });
    }

    // Vérification du nom de fichier standard Factur-X
    if metadata.xml_filename != "factur-x.xml" {
        warnings.push(format!(
            "Le nom de fichier XML '{}' n'est pas le nom standard 'factur-x.xml'",
            metadata.xml_filename
        ));
    }

    // Validation de la version Factur-X
    if metadata.facturx_version.is_empty() {
        errors.push(XmpValidationError {
            field: "facturx_version".to_string(),
            message: "La version Factur-X est requise".to_string(),
        });
    }

    XmpValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// Génère les métadonnées XMP conformes PDF/A-3 et Factur-X
///
/// Le XMP généré inclut :
/// - dc (Dublin Core) : titre, créateur, description
/// - xmp : dates de création et modification
/// - pdf : producteur
/// - pdfaid : conformité PDF/A-3
/// - fx : extension Factur-X
pub fn generate_xmp_metadata(metadata: &XmpMetadata) -> Result<String, String> {
    // Valider d'abord les métadonnées
    let validation = validate_xmp_metadata(metadata);
    if !validation.is_valid {
        let error_messages: Vec<String> = validation.errors.iter().map(|e| e.to_string()).collect();
        return Err(format!(
            "Validation XMP échouée: {}",
            error_messages.join("; ")
        ));
    }

    let now = Utc::now();
    let timestamp = now.format("%Y-%m-%dT%H:%M:%S+00:00").to_string();

    let xmp = format!(
        r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">

    <!-- Dublin Core -->
    <rdf:Description rdf:about=""
        xmlns:dc="http://purl.org/dc/elements/1.1/">
      <dc:format>application/pdf</dc:format>
      <dc:title>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">{title}</rdf:li>
        </rdf:Alt>
      </dc:title>
      <dc:creator>
        <rdf:Seq>
          <rdf:li>{author}</rdf:li>
        </rdf:Seq>
      </dc:creator>
      <dc:description>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">{subject}</rdf:li>
        </rdf:Alt>
      </dc:description>
    </rdf:Description>

    <!-- XMP Basic -->
    <rdf:Description rdf:about=""
        xmlns:xmp="http://ns.adobe.com/xap/1.0/">
      <xmp:CreatorTool>Generate-Factur-X</xmp:CreatorTool>
      <xmp:CreateDate>{timestamp}</xmp:CreateDate>
      <xmp:ModifyDate>{timestamp}</xmp:ModifyDate>
      <xmp:MetadataDate>{timestamp}</xmp:MetadataDate>
    </rdf:Description>

    <!-- PDF Properties -->
    <rdf:Description rdf:about=""
        xmlns:pdf="http://ns.adobe.com/pdf/1.3/">
      <pdf:Producer>Generate-Factur-X (printpdf + lopdf)</pdf:Producer>
    </rdf:Description>

    <!-- PDF/A Identification -->
    <rdf:Description rdf:about=""
        xmlns:pdfaid="http://www.aiim.org/pdfa/ns/id/">
      <pdfaid:part>3</pdfaid:part>
      <pdfaid:conformance>B</pdfaid:conformance>
    </rdf:Description>

    <!-- PDF/A Extension Schema for Factur-X -->
    <rdf:Description rdf:about=""
        xmlns:pdfaExtension="http://www.aiim.org/pdfa/ns/extension/"
        xmlns:pdfaSchema="http://www.aiim.org/pdfa/ns/schema#"
        xmlns:pdfaProperty="http://www.aiim.org/pdfa/ns/property#">
      <pdfaExtension:schemas>
        <rdf:Bag>
          <rdf:li rdf:parseType="Resource">
            <pdfaSchema:schema>Factur-X PDFA Extension Schema</pdfaSchema:schema>
            <pdfaSchema:namespaceURI>urn:factur-x:pdfa:CrossIndustryDocument:invoice:1p0#</pdfaSchema:namespaceURI>
            <pdfaSchema:prefix>fx</pdfaSchema:prefix>
            <pdfaSchema:property>
              <rdf:Seq>
                <rdf:li rdf:parseType="Resource">
                  <pdfaProperty:name>DocumentFileName</pdfaProperty:name>
                  <pdfaProperty:valueType>Text</pdfaProperty:valueType>
                  <pdfaProperty:category>external</pdfaProperty:category>
                  <pdfaProperty:description>Name of the embedded XML invoice file</pdfaProperty:description>
                </rdf:li>
                <rdf:li rdf:parseType="Resource">
                  <pdfaProperty:name>DocumentType</pdfaProperty:name>
                  <pdfaProperty:valueType>Text</pdfaProperty:valueType>
                  <pdfaProperty:category>external</pdfaProperty:category>
                  <pdfaProperty:description>INVOICE</pdfaProperty:description>
                </rdf:li>
                <rdf:li rdf:parseType="Resource">
                  <pdfaProperty:name>Version</pdfaProperty:name>
                  <pdfaProperty:valueType>Text</pdfaProperty:valueType>
                  <pdfaProperty:category>external</pdfaProperty:category>
                  <pdfaProperty:description>Version of the Factur-X standard</pdfaProperty:description>
                </rdf:li>
                <rdf:li rdf:parseType="Resource">
                  <pdfaProperty:name>ConformanceLevel</pdfaProperty:name>
                  <pdfaProperty:valueType>Text</pdfaProperty:valueType>
                  <pdfaProperty:category>external</pdfaProperty:category>
                  <pdfaProperty:description>Conformance level of the Factur-X invoice</pdfaProperty:description>
                </rdf:li>
              </rdf:Seq>
            </pdfaSchema:property>
          </rdf:li>
        </rdf:Bag>
      </pdfaExtension:schemas>
    </rdf:Description>

    <!-- Factur-X Specific Metadata -->
    <rdf:Description rdf:about=""
        xmlns:fx="urn:factur-x:pdfa:CrossIndustryDocument:invoice:1p0#">
      <fx:DocumentFileName>{xml_filename}</fx:DocumentFileName>
      <fx:DocumentType>INVOICE</fx:DocumentType>
      <fx:Version>{facturx_version}</fx:Version>
      <fx:ConformanceLevel>{profile_name}</fx:ConformanceLevel>
    </rdf:Description>

  </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#,
        title = escape_xml(&metadata.title),
        author = escape_xml(&metadata.author),
        subject = escape_xml(&metadata.subject),
        timestamp = timestamp,
        xml_filename = escape_xml(&metadata.xml_filename),
        facturx_version = escape_xml(&metadata.facturx_version),
        profile_name = metadata.profile.name(),
    );

    Ok(xmp)
}

/// Échappe les caractères spéciaux XML
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_xmp_metadata_valid() {
        let metadata = XmpMetadata {
            title: "Facture FA-2024-001".to_string(),
            author: "Ma Société".to_string(),
            subject: "Facture électronique".to_string(),
            profile: FacturXProfile::Minimum,
            xml_filename: "factur-x.xml".to_string(),
            facturx_version: "1.0".to_string(),
        };
        let result = validate_xmp_metadata(&metadata);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_xmp_metadata_missing_title() {
        let metadata = XmpMetadata {
            title: String::new(),
            author: "Ma Société".to_string(),
            ..Default::default()
        };
        let result = validate_xmp_metadata(&metadata);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.field == "title"));
    }

    #[test]
    fn test_validate_xmp_metadata_invalid_xml_extension() {
        let metadata = XmpMetadata {
            title: "Facture".to_string(),
            author: "Ma Société".to_string(),
            xml_filename: "factur-x.txt".to_string(),
            ..Default::default()
        };
        let result = validate_xmp_metadata(&metadata);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.field == "xml_filename"));
    }

    #[test]
    fn test_validate_xmp_metadata_non_standard_filename_warning() {
        let metadata = XmpMetadata {
            title: "Facture".to_string(),
            author: "Ma Société".to_string(),
            xml_filename: "invoice.xml".to_string(),
            ..Default::default()
        };
        let result = validate_xmp_metadata(&metadata);
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_generate_xmp_metadata() {
        let metadata = XmpMetadata {
            title: "Facture FA-2024-001".to_string(),
            author: "Ma Société".to_string(),
            subject: "Facture électronique".to_string(),
            profile: FacturXProfile::Minimum,
            xml_filename: "factur-x.xml".to_string(),
            facturx_version: "1.0".to_string(),
        };
        let xmp = generate_xmp_metadata(&metadata).unwrap();

        assert!(xmp.contains("pdfaid:part>3</pdfaid:part"));
        assert!(xmp.contains("pdfaid:conformance>B</pdfaid:conformance"));
        assert!(xmp.contains("fx:DocumentFileName>factur-x.xml</fx:DocumentFileName"));
        assert!(xmp.contains("fx:ConformanceLevel>MINIMUM</fx:ConformanceLevel"));
    }

    #[test]
    fn test_facturx_profile_urn() {
        assert_eq!(FacturXProfile::Minimum.urn(), "urn:factur-x.eu:1p0:minimum");
        assert_eq!(FacturXProfile::Basic.urn(), "urn:factur-x.eu:1p0:basic");
    }
}
