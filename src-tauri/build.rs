fn main() {
    let execution_level = if tauri_build::is_dev() { "asInvoker" } else { "requireAdministrator" };

    let manifest = format!(
        r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="{0}" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>"#,
        execution_level
    );

    let windows = tauri_build::WindowsAttributes::new().app_manifest(&manifest);
    tauri_build::try_build(
        tauri_build::Attributes::new().windows_attributes(windows),
    )
    .expect("failed to run build script");
}
