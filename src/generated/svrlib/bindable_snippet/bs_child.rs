/*
<script>
  use $components::bindable_snippet::BsChild;

  struct $state {
    snip: Option<$snippet(String)> = $bindable(None), // becomes MutateTracker<Option<Box<dyn SnippetFactoryTrait<(DynamicArg<String>,)>>>>
  }
</script>

<#snippet my_snip(text: String)>
  <p>The message: {text}</p>
</snippet>
*/
