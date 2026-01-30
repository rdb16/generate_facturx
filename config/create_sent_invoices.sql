-- Création de la base de données K_Factur_x (si nécessaire)
-- CREATE DATABASE "k_factur_x";

-- Table des factures envoyées
CREATE TABLE IF NOT EXISTS sent_invoices (
    invoice_num     VARCHAR(50) PRIMARY KEY,
    company_name    VARCHAR(255) NOT NULL,
    company_siret   VARCHAR(14) NOT NULL,
    xml_facture     XML NOT NULL,
    pdf_path        VARCHAR(500) NOT NULL,
    invoice_date    DATE NOT NULL,
    created_at      TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    validated_at    TIMESTAMP WITH TIME ZONE DEFAULT NULL
);

-- Index pour améliorer les recherches
CREATE INDEX IF NOT EXISTS idx_sent_invoices_company_siret ON sent_invoices(company_siret);
CREATE INDEX IF NOT EXISTS idx_sent_invoices_invoice_date ON sent_invoices(invoice_date);
CREATE INDEX IF NOT EXISTS idx_sent_invoices_company_name ON sent_invoices(company_name);

-- Commentaires sur la table et les colonnes
COMMENT ON TABLE sent_invoices IS 'Table des factures Factur-X générées et envoyées';
COMMENT ON COLUMN sent_invoices.invoice_num IS 'Numéro unique de la facture (clé primaire)';
COMMENT ON COLUMN sent_invoices.company_name IS 'Nom de la société facturée';
COMMENT ON COLUMN sent_invoices.company_siret IS 'SIRET de la société facturée (14 chiffres)';
COMMENT ON COLUMN sent_invoices.xml_facture IS 'Contenu XML Factur-X embarqué dans le PDF';
COMMENT ON COLUMN sent_invoices.pdf_path IS 'Chemin de stockage du fichier PDF';
COMMENT ON COLUMN sent_invoices.invoice_date IS 'Date de facturation';
COMMENT ON COLUMN sent_invoices.created_at IS "Date de création de l'enregistrement";
COMMENT ON COLUMN sent_invoices.validated_at IS 'Date de validation du XML contre le schéma Factur-X (NULL si non validé)';
