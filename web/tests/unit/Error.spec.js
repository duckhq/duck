import { shallowMount } from '@vue/test-utils'
import { data } from "@/js/store"
import Error from '@/Error.vue'

jest.mock("@/js/store")

const mount = function() {
  return shallowMount(Error, {
    stubs: [
      'fa-icon'
    ]
  })
}

describe('Error.vue', () => {
  beforeEach(() => {
    data.server = ""
  })

  it('shows server host in error message when present', () => {
    data.server = "localhost:12345"
    const wrapper = mount()
    
    expect(wrapper.text()).toContain(`The Duck server could not be reached at "${data.server}".`)
  })

  it('shows a different error message when server host not supplied', () => {
    const wrapper = mount()

    expect(wrapper.text()).toContain("The local Duck server could not be reached.")
  })
})
