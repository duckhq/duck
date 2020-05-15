<template>
  <fa-icon class="font-semibold" :icon="getClass()" fixed-width />
</template>

<script>
import { library } from "@fortawesome/fontawesome-svg-core";
import {
  faCheckCircle,
  faRunning,
  faExclamationTriangle, 
  faStopCircle,
  faQuestionCircle,
  faClock 
} from "@fortawesome/free-solid-svg-icons";

library.add(
  faCheckCircle,
  faRunning,
  faExclamationTriangle, 
  faStopCircle,
  faQuestionCircle,
  faClock);

export default {
  props: ["build"],
  methods: {
      getClass: function() {
          switch(this.build.status) {
              case "Success": return "check-circle";
              case "Failed": return "exclamation-triangle";
              case "Running": return "running";
              case "Canceled": return "stop-circle";
              case "Queued": return "clock";
              case "Skipped": return "stop-circle";
              default: return "question-circle"
          }
      }
  }
};
</script>