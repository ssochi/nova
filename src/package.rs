#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportedPackage {
    Fmt,
}

impl ImportedPackage {
    pub fn import_path(self) -> &'static str {
        match self {
            ImportedPackage::Fmt => "fmt",
        }
    }

    pub fn binding_name(self) -> &'static str {
        match self {
            ImportedPackage::Fmt => "fmt",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageFunction {
    FmtPrint,
    FmtPrintln,
    FmtSprint,
}

impl PackageFunction {
    pub fn render(self) -> &'static str {
        match self {
            PackageFunction::FmtPrint => "fmt.Print",
            PackageFunction::FmtPrintln => "fmt.Println",
            PackageFunction::FmtSprint => "fmt.Sprint",
        }
    }
}
