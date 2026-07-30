/*
<script>
  use $components::bindable_snippet::BsChild;

  struct $state {
    snip: Option<$snippet(String)> = $state(None), // becomes MutateTracker<Option<Box<dyn SnippetFactoryTrait<(DynamicArg<String>,)>>>>
    message: String,
  }
</script>

<input bind:value={message} />
<BsChild bind:{snip}>

{@render snip(message)}
*/
