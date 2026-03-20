#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportedPackage {
    Fmt,
    Strings,
    Bytes,
}

impl ImportedPackage {
    pub fn import_path(self) -> &'static str {
        match self {
            ImportedPackage::Fmt => "fmt",
            ImportedPackage::Strings => "strings",
            ImportedPackage::Bytes => "bytes",
        }
    }

    pub fn default_binding_name(self) -> &'static str {
        match self {
            ImportedPackage::Fmt => "fmt",
            ImportedPackage::Strings => "strings",
            ImportedPackage::Bytes => "bytes",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageFunction {
    FmtPrint,
    FmtPrintln,
    FmtSprint,
    StringsContains,
    StringsHasPrefix,
    StringsCut,
    StringsCutPrefix,
    StringsCutSuffix,
    StringsJoin,
    StringsRepeat,
    BytesEqual,
    BytesContains,
    BytesHasPrefix,
    BytesCut,
    BytesCutPrefix,
    BytesCutSuffix,
    BytesJoin,
    BytesRepeat,
}

impl PackageFunction {
    pub fn render(self) -> &'static str {
        match self {
            PackageFunction::FmtPrint => "fmt.Print",
            PackageFunction::FmtPrintln => "fmt.Println",
            PackageFunction::FmtSprint => "fmt.Sprint",
            PackageFunction::StringsContains => "strings.Contains",
            PackageFunction::StringsHasPrefix => "strings.HasPrefix",
            PackageFunction::StringsCut => "strings.Cut",
            PackageFunction::StringsCutPrefix => "strings.CutPrefix",
            PackageFunction::StringsCutSuffix => "strings.CutSuffix",
            PackageFunction::StringsJoin => "strings.Join",
            PackageFunction::StringsRepeat => "strings.Repeat",
            PackageFunction::BytesEqual => "bytes.Equal",
            PackageFunction::BytesContains => "bytes.Contains",
            PackageFunction::BytesHasPrefix => "bytes.HasPrefix",
            PackageFunction::BytesCut => "bytes.Cut",
            PackageFunction::BytesCutPrefix => "bytes.CutPrefix",
            PackageFunction::BytesCutSuffix => "bytes.CutSuffix",
            PackageFunction::BytesJoin => "bytes.Join",
            PackageFunction::BytesRepeat => "bytes.Repeat",
        }
    }
}
