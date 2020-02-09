<template>
  <div>
    <nav class="main-nav">
      <Burger></Burger>
    </nav>
    <Sidebar>
      <ul class="sidebar-panel-nav">
        <li>
          <a href="/" :class="{ 'active' : isDefaultView() }">Everything</a>
        </li>
        <li v-for="view in this.get_views()" :key="view.id">
          <a :class="{ 'active' : isActive(view) }" :href="get_view_url(view)">{{ view.name }}</a>
        </li>
      </ul>
    </Sidebar>
  </div>
</template>

<script>
import Burger from "./Menu/Burger.vue";
import Sidebar from "./Menu/Sidebar.vue";

export default {
  props: ["serverInfo", "view"],
  components: {
    Burger,
    Sidebar
  },
  computed: {
    hasViews() {
      if (this.serverInfo != undefined) {
        return this.serverInfo.views.length > 0;
      }
      return false;
    }
  },
  methods: {
    isDefaultView: function() {
      return this.view == undefined;
    },
    isActive: function(view) {
      if (this.view == view.slug) {
        return true;
      }
      return false;
    },
    get_views: function() {
      if (this.serverInfo != undefined) {
        return this.serverInfo.views;
      }
      return [];
    },
    get_view_url: function(view) {
      return "/?view=" + view.slug;
    }
  }
};
</script>

<style scoped>
.main-nav {
  display: block;
  width: 1px;
  float: right;
  justify-content: space-between;
  padding-right: 48px;
  padding-top: 8px;
}

ul.sidebar-panel-nav {
  list-style-type: none;
}

ul.sidebar-panel-nav > li > a {
  color: #bbb;
  text-decoration: none;
  font-size: 1.5rem;
  display: block;
  padding-bottom: 0.5em;
}

ul.sidebar-panel-nav > li > a.active {
  color: #fff;
  font-weight: bold;
}

ul.sidebar-panel-nav > li > a:hover {
  color: #fff;
}
</style>