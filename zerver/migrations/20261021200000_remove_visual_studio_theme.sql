-- Visual Studio left the theme list for Flexoki; its users move to VS Code, the closest look.
UPDATE user_preferences SET theme = 'vscode' WHERE theme = 'visual-studio';
