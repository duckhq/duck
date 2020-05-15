jest.mock("@/js/store", () => ({
  data: {}
}))

import { shallowMount } from "@vue/test-utils"
import ViewList from "@/components/ViewList.vue"
import Settings from "@/components/Settings.vue"

describe("Settings", () => {
  let wrapper = null
  
  function mount() {
    wrapper = shallowMount(Settings, {
      stubs: [
        "fa-icon"
      ],
      components: {
        ViewList
      }
    })
  }

  it("should initialize to the 'views' page", () => {
    mount()
    expect(wrapper.vm.$data.current).toBe('views')
  })

  it("should show views page when views button clicked", async () => {
    mount()
    wrapper.find("button#info").trigger("click")
    wrapper.find("button#views").trigger("click")

    await wrapper.vm.$nextTick()

    expect(wrapper.vm.show_views).toBe(true)
    expect(wrapper.vm.show_info).toBe(false)
    expect(wrapper.find("ViewList-stub").exists()).toBeTruthy()
    expect(wrapper.find("ServerInfo-stub").exists()).toBe(false)
  })

  it("should show info page when info button clicked", async () => {
    mount()
    wrapper.find("button#info").trigger("click")

    await wrapper.vm.$nextTick()
    
    expect(wrapper.vm.show_info).toBe(true)
    expect(wrapper.vm.show_views).toBe(false)
    expect(wrapper.find("ServerInfo-stub").exists()).toBeTruthy()
    expect(wrapper.find("ViewList-stub").exists()).toBe(false)
  })

  it("should emit 'close' event when escape key pressed", async () => {
    mount()
    const modal = wrapper.find({ ref: "modal" })
    modal.trigger('keyup.esc')
    await wrapper.vm.$nextTick()
    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it("should emit 'close' event when view list data changes", async () => {
    mount()
    const viewList = wrapper.find("ViewList-stub")
    viewList.vm.$emit("view_changed")
    await wrapper.vm.$nextTick()
    expect(wrapper.emitted('close')).toBeTruthy()
  })

})
