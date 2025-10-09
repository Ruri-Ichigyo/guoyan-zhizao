describe('Fruit Sorting App', () => {
  beforeEach(() => {
    cy.intercept('GET', '/api/v1/statistics', { fixture: 'statistics-data.json' }).as('getStatistics')
    cy.intercept('GET', '/api/v1/results*', { fixture: 'results-data.json' }).as('getResults')
    cy.visit('/')
  })

  it('shows home hero content', () => {
    cy.contains('果蔬智造核心平台').should('be.visible')
    cy.contains('上传识别 · 智能分析 · 快速查询').should('be.visible')
    cy.get('header .app-nav').within(() => {
      cy.contains('首页').should('have.class', 'router-link-active')
      cy.contains('图像上传').should('exist')
      cy.contains('检测结果').should('exist')
    })
  })

  it('navigates to upload view and shows upload panel', () => {
    cy.contains('图像上传').click()
    cy.url().should('include', '/upload')
    cy.contains('图像上传').should('be.visible')
    cy.contains('上传果蔬图片并触发 AI 分析').should('be.visible')
    cy.get('.upload-placeholder').should('be.visible')
  })

  it('renders results view with mocked data', () => {
    cy.contains('检测结果').click()
    cy.url().should('include', '/results')

    cy.wait('@getStatistics')
    cy.wait('@getResults')

    cy.contains('历史记录').should('be.visible')
    cy.contains('apple_batch_001.jpg').should('be.visible')
    cy.contains('orange_batch_002.jpg').should('be.visible')
  })
})
