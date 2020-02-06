<template>
  <div id="app">
    <div id="builds">
      <vue-headful :title="title" />
      <section v-if="errored && !docker">
        <p>
          The Duck server could not be reached at {{ this.address }}. Retrying...
          <pulse-loader color="#DDDDDD" size="8px" style="text-align: left;" />
        </p>
      </section>
      <section v-else-if="errored && docker">
        <p>
          The Duck server could not be reached. Retrying...
          <pulse-loader color="#DDDDDD" size="8px" style="text-align: left;" />
        </p>
      </section>
      <section v-else-if="loading">
        <p>Loading...</p>
      </section>
      <section v-else>
        <builds :builds="allBuilds" />
      </section>
      <vue-progress-bar style="z-index:-1"></vue-progress-bar>
    </div>
  </div>
</template>

<script>
import axios from "axios";
import builds from "./components/Builds.vue";
import PulseLoader from "vue-spinner/src/PulseLoader.vue";

export default {
  name: "App",
  components: {
    Builds: builds,
    PulseLoader
  },
  data() {
    return {
      address: process.env.VUE_APP_MY_DUCK_SERVER,
      docker: process.env.VUE_APP_MY_DUCK_SERVER == "",
      view: this.$route.query.view,
      serverInfo: null,
      builds: null,
      loading: true,
      errored: false
    };
  },
  computed: {
    title() {
      if (this.serverInfo == null) {
        return "Duck";
      } else {
        if (this.view != undefined) {
          for (var i=0; i < this.serverInfo.views.length; i++) {
              if (this.serverInfo.views[i].slug === this.view) {
                  return this.serverInfo.views[i].name;
              }
          }
        }
        return this.serverInfo.title;
      }
    },
    allBuilds() {
      return this.builds
        .slice()
        .sort((a, b) => (a.started < b.started ? 1 : -1));
    }
  },
  methods: {
    loadData: function() {
      this.$Progress.start();

      let address = this.address + "/api/builds";
      if (this.view != undefined) {
        address = address + "/view/" + this.view;
      }

      axios
        .get(address)
        .then(response => {
          this.builds = response.data;
          this.errored = false;
          this.$Progress.finish();

          if (this.serverInfo == null) {
            this.updateServerInfo();
          }
        })
        .catch(() => {
          this.errored = true;
          this.$Progress.fail();
        })
        .finally(() => (this.loading = false));
    },
    updateServerInfo: function() {
      axios
        .get(this.address + "/api/server")
        .then(response => {
          this.serverInfo = response.data;
        })
        .catch(() => {
          this.serverInfo = null;
        });
    }
  },
  mounted() {
    this.loadData();
    setInterval(
      function() {
        this.loadData();
      }.bind(this),
      3000
    );
  }
};
</script>

<style scoped>
#title {
  padding-bottom: 5px;
}
#builds {
  padding-top: 20px;
  padding-left: 20px;
  padding-right: 20px;
}
</style>