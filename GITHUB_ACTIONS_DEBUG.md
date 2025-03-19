# Debugging GitHub Actions Deployment

This document provides troubleshooting steps if you encounter issues with GitHub Actions deployment.

## Common Issues and Solutions

### Missing download info for actions/upload-artifact@v3

This error occurs when there's a version mismatch or the action is not available. To fix this:

1. Update to the latest version of the action in `.github/workflows/deploy.yml`
2. If you're using a specific version like v3, try changing to a more stable version like v2
3. Check the GitHub Status page to see if there are any ongoing issues with GitHub Actions

### WebGL or WebAssembly Rendering Issues

If the cube is not rendering correctly:

1. Open browser developer tools (F12)
2. Check for any JavaScript errors in the console
3. Look for specific error messages that might indicate what's failing
4. Try a different browser, as WebGL support varies

## Diagnosing Deployment Failures

When your GitHub Actions workflow fails:

1. Go to your repository on GitHub
2. Click on the "Actions" tab
3. Find the failed workflow run
4. Click on the job that failed
5. Expand the step that failed to see detailed logs
6. Look for error messages that provide clues

## Testing Locally Before Deployment

It's always a good idea to test locally before pushing:

```bash
# Build the WebAssembly package
wasm-pack build --target web

# Start a local server to test
python -m http.server
# or
npx serve

# Navigate to http://localhost:8000 in your browser
```

## GitHub Pages Deployment Specifics

For GitHub Pages to work correctly:

1. Make sure the branch in your workflow file matches the branch you're working on
2. Ensure the `permissions` section is correctly set up
3. Verify that GitHub Pages is enabled in your repository settings
4. Check that the path used in the upload-pages-artifact step is correct

## Getting Help

If you still encounter issues:

1. Open an issue in the repository with details about what's happening
2. Include screenshots of any error messages
3. Share browser console logs
4. Mention what steps you've already tried

Remember that GitHub Actions and GitHub Pages might take a few minutes to reflect your changes after deployment. 