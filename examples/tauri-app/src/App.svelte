<script>
  import { getProgramByPath, getProgramList } from 'tauri-plugin-shion-watcher-api'

 const createIconBlob = (buffer) => new Blob([new Uint8Array(buffer)], { type: 'image/png' })

  let programList = []
  
  ;(async () => {
    const list = await getProgramList()
    const program = await getProgramByPath("C:\\WINDOWS\\System32\\WindowsPowerShell\\v1.0\\powershell.exe")
    programList = [program, ...list].map(p => {
      return {
        ...p,
        icon: URL.createObjectURL(createIconBlob(p.icon))
      }
    })
  })()

</script>

<main class="container">
<div>
  {#each programList as { path, name, icon }}
    <div>
      <div>{path}</div>
      <div>{name}</div>
      <img src={icon} alt={name} />
    </div>
  {/each}
</div>
</main>

<style>

</style>
