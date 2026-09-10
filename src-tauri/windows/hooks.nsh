; ServerForge NSIS installer hooks

!macro NSIS_HOOK_PREINSTALL
  DetailPrint "Preparing ServerForge..."
!macroend

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "ServerForge installed. Shortcuts were created."
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Removing ServerForge..."
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DetailPrint "ServerForge has been removed."
!macroend
