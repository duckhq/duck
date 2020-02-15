<template>
  <div class="ui link card" :class="getBackgroundClass()" @click="navigateToBuild()">
    <div class="content">
      <!-- Project -->
      <div class="header">
        <span class="project">{{ build.project }}</span>
        <BuildIcon :build="build" class="right floated" />
      </div>
      <div class="description">
        <!-- Project -->
        <div class="item">
          <i class="hashtag icon"></i>
          {{ build.build }}
        </div>
        <!-- Branch -->
        <div class="item">
          <i class="code branch icon"></i>
          {{ build.branch }}
        </div>
        <!-- Build number -->
        <div class="item">
          <StatusIcon :build="build" />
          {{ getBuildKind() }} {{ build.buildNumber }}
        </div>
      </div>
    </div>
    <div class="extra content">
      <span>{{ getStatusText() }}&nbsp;</span>
      <span v-if="build.status != 'Running'">{{ build.finished | moment("from", "now") }}</span>
      <span v-if="build.status == 'Running'">{{ build.started | moment("from", "now") }}</span>
    </div>
  </div>
</template>

<script>
import BuildIcon from "./BuildIcon.vue";
import StatusIcon from "./StatusIcon.vue";

export default {
  props: ["build"],
  components: {
    BuildIcon,
    StatusIcon
  },
  methods: {
    navigateToBuild: function() {
      window.open(this.build.url, "_blank");
    },
    getBackgroundClass: function() {
      return this.build.status.toLowerCase();
    },
    getBuildKind: function() {
      switch (this.build.provider) {
        case "OctopusDeploy":
          return "Deployment";
        default:
          return "Build";
      }
    },
    getStatusText: function() {
      switch (this.build.status) {
        case "Success":
          return "Succeeded";
        case "Running":
          return "Started";
        default:
          return this.build.status;
      }
    }
  }
};
</script>

<style scoped>
.project {
  padding-right: 32px;
}

.success {
  background-color: #66bb6a !important;
}

.failed {
  background-color: #d84b4b !important;
}

.running {
  background-color: #29b6f6 !important;
}

.canceled {
  background-color: #999999 !important;
}

.queued {
  background-color: #999999 !important;
}
</style>