Unicode true
RequestExecutionLevel user
SetCompressor /SOLID lzma

!include "MUI2.nsh"

!ifndef APP_VERSION
	!error "APP_VERSION must be provided with -DAPP_VERSION=..."
!endif
!ifndef SOURCE_EXE
	!error "SOURCE_EXE must be provided with -DSOURCE_EXE=..."
!endif
!ifndef WEBVIEW_DLL
	!error "WEBVIEW_DLL must be provided with -DWEBVIEW_DLL=..."
!endif
!ifndef OUTPUT_FILE
	!error "OUTPUT_FILE must be provided with -DOUTPUT_FILE=..."
!endif
!ifndef APP_ICON
	!error "APP_ICON must be provided with -DAPP_ICON=..."
!endif

!define PRODUCT_NAME "Migurinth"
!define PRODUCT_PUBLISHER "BreezeDelegate"
!define PRODUCT_WEB "https://github.com/BreezeDelegate/migurinth"
!define INSTALL_KEY "Software\Migurinth"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\Migurinth"

Name "${PRODUCT_NAME}"
OutFile "${OUTPUT_FILE}"
InstallDir "$LOCALAPPDATA\Programs\Migurinth"
InstallDirRegKey HKCU "${INSTALL_KEY}" "InstallDir"
Icon "${APP_ICON}"
UninstallIcon "${APP_ICON}"
BrandingText "Migurinth"

!define MUI_ABORTWARNING
!define MUI_ICON "${APP_ICON}"
!define MUI_UNICON "${APP_ICON}"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "Migurinth" SEC_MAIN
	SetShellVarContext current
	SetOutPath "$INSTDIR"

	File /oname=Migurinth.exe "${SOURCE_EXE}"
	File /oname=WebView2Loader.dll "${WEBVIEW_DLL}"
	WriteUninstaller "$INSTDIR\Uninstall.exe"

	WriteRegStr HKCU "${INSTALL_KEY}" "InstallDir" "$INSTDIR"
	WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${PRODUCT_NAME}"
	WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${APP_VERSION}"
	WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
	WriteRegStr HKCU "${UNINSTALL_KEY}" "URLInfoAbout" "${PRODUCT_WEB}"
	WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\Migurinth.exe"
	WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
	WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
	WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
	WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1

	CreateDirectory "$SMPROGRAMS\Migurinth"
	CreateShortcut "$SMPROGRAMS\Migurinth\Migurinth.lnk" "$INSTDIR\Migurinth.exe"
	CreateShortcut "$DESKTOP\Migurinth.lnk" "$INSTDIR\Migurinth.exe"

	WriteRegStr HKCU "Software\Classes\.mrpack" "" "Migurinth.mrpack"
	WriteRegStr HKCU "Software\Classes\Migurinth.mrpack" "" "Modrinth Modpack"
	WriteRegStr HKCU "Software\Classes\Migurinth.mrpack\DefaultIcon" "" "$INSTDIR\Migurinth.exe,0"
	WriteRegStr HKCU "Software\Classes\Migurinth.mrpack\shell\open\command" "" '"$INSTDIR\Migurinth.exe" "%1"'

	WriteRegStr HKCU "Software\Classes\modrinth" "" "URL:Modrinth Protocol"
	WriteRegStr HKCU "Software\Classes\modrinth" "URL Protocol" ""
	WriteRegStr HKCU "Software\Classes\modrinth\DefaultIcon" "" "$INSTDIR\Migurinth.exe,0"
	WriteRegStr HKCU "Software\Classes\modrinth\shell\open\command" "" '"$INSTDIR\Migurinth.exe" "%1"'

	System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
SectionEnd

Section "Uninstall"
	SetShellVarContext current

	Delete "$DESKTOP\Migurinth.lnk"
	Delete "$SMPROGRAMS\Migurinth\Migurinth.lnk"
	RMDir "$SMPROGRAMS\Migurinth"

	DeleteRegKey HKCU "Software\Classes\Migurinth.mrpack"
	DeleteRegValue HKCU "Software\Classes\.mrpack" ""
	DeleteRegKey HKCU "Software\Classes\modrinth"
	DeleteRegKey HKCU "${UNINSTALL_KEY}"
	DeleteRegKey HKCU "${INSTALL_KEY}"

	Delete "$INSTDIR\Migurinth.exe"
	Delete "$INSTDIR\WebView2Loader.dll"
	Delete "$INSTDIR\Uninstall.exe"
	RMDir "$INSTDIR"

	System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
SectionEnd
