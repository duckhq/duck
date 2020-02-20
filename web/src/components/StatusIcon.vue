<template>
  <i class="icon" :class="getClass()"></i>
</template>

<script>
export default {
  props: ["build"],
  methods: {
      getClass: function() {
          switch(this.build.status) {
              case "Success": return "circle check";
              case "Failed": return "triangle exclamation";
              case "Running": return "cog loading";
              case "Canceled": return "circle times";
              case "Queued": return "clock";
              default: return "question circle"

          }
      }
  }
};
</script>