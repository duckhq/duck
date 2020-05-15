jest.mock("@/js/store", () => ({
  data: {}
}))

import { createLocalVue, shallowMount } from "@vue/test-utils"
import VueList from "@/components/ViewList.vue"
import { data } from "@/js/store"
import VueRouter from "vue-router"

const localVue = createLocalVue()
localVue.use(VueRouter)
const router = new VueRouter()

describe("Vue List", () => {
  let wrapper = null

  const title = "Hello!"
  const view1Name = "View 1"
  const view2Name = "View 2"

  function mockView() {
    data.server = "localhost"
    data.info = {
      title,
      views: [ 
        {
          name: view1Name,
          slug: "/view1"
        },
        {
          name: view2Name
        }
      ]
    }
  }

  function mount() {
    router.push = jest.fn()
    wrapper = shallowMount(VueList, {
      stubs: [
        "fa-icon"
      ],
      localVue,
      router
    })
  }

  it("display message if no views available", () => {
    mount()
    expect(wrapper.text()).toBe("No views available")
  })

  it("displays active view button when at least one view available", () => {
    mockView()
    mount()
    const viewButtons = wrapper.findAll("button")
    expect(viewButtons.at(0).text()).toBe(title)
    expect(viewButtons.at(1).text()).toBe(view1Name)
    expect(viewButtons.at(2).text()).toBe(view2Name)
  })

  it("emits 'view_changed' event when a view is clicked", async () => {
    mockView()
    mount()
    const viewButtons = wrapper.findAll("button")
    const view1Button = viewButtons.at(1)
    view1Button.trigger("click")
    await wrapper.vm.$nextTick()
    wrapper.emitted("view_changed")
  })

  it("routes when a view is clicked", async () => {
    mockView()
    mount()
    const viewButtons = wrapper.findAll("button")
    const view1Button = viewButtons.at(1)
    view1Button.trigger("click")
    await wrapper.vm.$nextTick()
    expect(wrapper.vm.$router.push).toHaveBeenCalledWith("/?view=/view1&server=localhost")
  })

  it("routes when first view is clicked", async () => {
    mockView()
    mount()
    const viewButtons = wrapper.findAll("button")
    const view1Button = viewButtons.at(0)
    view1Button.trigger("click")
    await wrapper.vm.$nextTick()
    expect(wrapper.vm.$router.push).toHaveBeenCalledWith("?server=localhost")
  })
})