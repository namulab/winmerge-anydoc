$ErrorActionPreference = "Stop"

# このスクリプトの2階層上をプロジェクトルートとする
$ProjectRoot = (
    Resolve-Path (
        Join-Path $PSScriptRoot "..\.."
    )
).Path

# Cargoが生成するDebug版COM DLL
$DllPath = (
    Resolve-Path (
        Join-Path `
            $ProjectRoot `
            "target\debug\compare_office_with_anydoc.dll"
    )
).Path

# このテスト専用のOffice文書
$InputFile = (
    Resolve-Path (
        Join-Path `
            $PSScriptRoot `
            "test-com-plugin.docx"
    )
).Path

# テスト成果物はプロジェクト配下に残さずTEMPへ作成する
$OutputFile = Join-Path `
    ([System.IO.Path]::GetTempPath()) `
    ("compare-office-with-anydoc-{0}.md" -f [Guid]::NewGuid())

if (-not ("WinMergeAnydocComSmoke" -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class WinMergeAnydocComSmoke
{
    [DllImport(
        "kernel32.dll",
        CharSet = CharSet.Unicode,
        SetLastError = true)]
    private static extern IntPtr LoadLibraryW(
        string lpFileName);

    [DllImport(
        "kernel32.dll",
        SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool FreeLibrary(
        IntPtr hModule);

    [DllImport(
        "kernel32.dll",
        CharSet = CharSet.Ansi,
        SetLastError = true)]
    private static extern IntPtr GetProcAddress(
        IntPtr hModule,
        string lpProcName);

    [UnmanagedFunctionPointer(
        CallingConvention.StdCall)]
    private delegate int DllGetClassObjectDelegate(
        ref Guid rclsid,
        ref Guid riid,
        out IntPtr ppv);

    [ComImport]
    [Guid("00000001-0000-0000-C000-000000000046")]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    private interface IClassFactory
    {
        [PreserveSig]
        int CreateInstance(
            IntPtr pUnkOuter,
            ref Guid riid,
            out IntPtr ppvObject);

        [PreserveSig]
        int LockServer(
            [MarshalAs(UnmanagedType.Bool)]
            bool fLock);
    }

    public static string Run(
        string dllPath,
        string inputFile,
        string outputFile)
    {
        IntPtr module = LoadLibraryW(dllPath);

        if (module == IntPtr.Zero)
        {
            throw new InvalidOperationException(
                "LoadLibraryW failed: " +
                Marshal.GetLastWin32Error());
        }

        try
        {
            IntPtr proc = GetProcAddress(
                module,
                "DllGetClassObject");

            if (proc == IntPtr.Zero)
            {
                throw new InvalidOperationException(
                    "DllGetClassObject was not found.");
            }

            var getClassObject =
                Marshal.GetDelegateForFunctionPointer<
                    DllGetClassObjectDelegate>(proc);

            Guid clsid =
                new Guid(
                    "A4F619AC-6B9E-48A9-A09D-3B12200693EA");

            Guid iidClassFactory =
                new Guid(
                    "00000001-0000-0000-C000-000000000046");

            IntPtr factoryPtr;

            int hr = getClassObject(
                ref clsid,
                ref iidClassFactory,
                out factoryPtr);

            Marshal.ThrowExceptionForHR(hr);

            IClassFactory factory =
                (IClassFactory)
                Marshal.GetObjectForIUnknown(factoryPtr);

            Marshal.Release(factoryPtr);

            Guid iidDispatch =
                new Guid(
                    "00020400-0000-0000-C000-000000000046");

            IntPtr dispatchPtr;

            hr = factory.CreateInstance(
                IntPtr.Zero,
                ref iidDispatch,
                out dispatchPtr);

            Marshal.ThrowExceptionForHR(hr);

            Marshal.FinalReleaseComObject(factory);

            dynamic plugin =
                Marshal.GetObjectForIUnknown(dispatchPtr);

            Marshal.Release(dispatchPtr);

            try
            {
                string pluginEvent =
                    (string)plugin.PluginEvent;

                string description =
                    (string)plugin.PluginDescription;

                string filters =
                    (string)plugin.PluginFileFilters;

                bool automatic =
                    (bool)plugin.PluginIsAutomatic;

                string extension =
                    (string)plugin.PluginUnpackedFileExtension;

                bool changed = false;
                int subcode = 0;

                bool success =
                    (bool)plugin.UnpackFile(
                        inputFile,
                        outputFile,
                        ref changed,
                        ref subcode);

                if (!success)
                {
                    throw new InvalidOperationException(
                        "UnpackFile returned false.");
                }

                return
                    "PluginEvent=" + pluginEvent +
                    Environment.NewLine +
                    "PluginDescription=" + description +
                    Environment.NewLine +
                    "PluginFileFilters=" + filters +
                    Environment.NewLine +
                    "PluginIsAutomatic=" + automatic +
                    Environment.NewLine +
                    "PluginUnpackedFileExtension=" + extension +
                    Environment.NewLine +
                    "UnpackFile.Success=" + success +
                    Environment.NewLine +
                    "UnpackFile.Changed=" + changed +
                    Environment.NewLine +
                    "UnpackFile.Subcode=" + subcode;
            }
            finally
            {
                Marshal.FinalReleaseComObject(plugin);
            }
        }
        finally
        {
            FreeLibrary(module);
        }
    }
}
'@ -ReferencedAssemblies "Microsoft.CSharp.dll"
}

try {
    $Result = [WinMergeAnydocComSmoke]::Run(
        $DllPath,
        $InputFile,
        $OutputFile
    )

    $Result

    if (-not (Test-Path $OutputFile)) {
        throw "UnpackFile succeeded, but the output file was not created."
    }

    $File = Get-Item $OutputFile

    if ($File.Length -eq 0) {
        throw "The generated Markdown file is empty."
    }

    ""
    "Output verified:"
    "  Size=$($File.Length) bytes"
    "  Temporary file will be deleted."
}
finally {
    if (Test-Path $OutputFile) {
        Remove-Item $OutputFile -Force
    }
}
