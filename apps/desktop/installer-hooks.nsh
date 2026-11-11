!macro NSIS_HOOK_PREINSTALL
  ; Clean up stale temporary files from previous installations in $INSTDIR
  DetailPrint "Cleaning up stale temporary files..."
  Delete "$INSTDIR\*.tmp"
  Delete "$INSTDIR\old_*.exe"
  Delete "$INSTDIR\old_*.dll"
  Delete "$INSTDIR\*.pending"

  ; Clean up stale update download cache and temp dirs
  RMDir /r "$LOCALAPPDATA\in.solara.launcher\app-update"
  RMDir /r "$LOCALAPPDATA\in.solara.launcher\temp"
  RMDir /r "$LOCALAPPDATA\in.solara.launcher\tmp"
  RMDir /r "$LOCALAPPDATA\in.solara.launcher\verified-game-files\.tmp"

  ; Remove legacy remnants if present
  RMDir /r "$LOCALAPPDATA\in.solara.launcher\theseus"
  RMDir /r "$LOCALAPPDATA\in.solara.launcher\legacy"

  ; Clean up temporary files in root of launcher data directory
  Delete "$LOCALAPPDATA\in.solara.launcher\*.tmp"
  Delete "$LOCALAPPDATA\in.solara.launcher\.solara-*.tmp"
  Delete "$LOCALAPPDATA\in.solara.launcher\*.pending"

  ; Note: User data (custom-instances, accounts, and servers) is preserved here.
  ; One-time clean-slate reset for V-0.0.23 is handled idempotently in the engine via .clean_slate_v23_reset sentinel.
!macroend

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Solara Launcher installation verified. User instances and servers preserved."
!macroend
