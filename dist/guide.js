document.querySelectorAll('.copy-command').forEach(button => {
  button.addEventListener('click', async () => {
    const block=button.closest('.command');
    const code=block.querySelector('code');
    const status=block.querySelector('.copy-result');
    try {
      await navigator.clipboard.writeText(code.textContent);
      status.textContent='Copied';
    } catch {
      const range=document.createRange();range.selectNodeContents(code);
      const selection=window.getSelection();selection.removeAllRanges();selection.addRange(range);
      status.textContent='Selected — copy the text manually';
    }
  });
});
