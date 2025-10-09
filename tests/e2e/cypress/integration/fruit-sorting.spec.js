describe('E2E Tests for Fruit Sorting System', () => {
  beforeEach(() => {
    // Visit the application
    cy.visit('/')

    // Mock authentication
    cy.window().then((win) => {
      win.localStorage.setItem('auth-token', 'mock-jwt-token')
    })
  })

  describe('Upload Panel', () => {
    it('should allow file upload', () => {
      // Navigate to upload page
      cy.get('[data-cy=nav-upload]').click()

      // Upload a test image
      cy.get('[data-cy=file-upload]').selectFile('cypress/fixtures/test-apple.jpg')

      // Verify upload success
      cy.get('[data-cy=upload-success]').should('be.visible')
      cy.get('[data-cy=upload-id]').should('contain', 'upload_')
    })

    it('should show upload progress', () => {
      cy.get('[data-cy=nav-upload]').click()

      // Mock slow upload
      cy.intercept('POST', '/api/v1/uploads', {
        delay: 2000,
        body: { upload_id: 'test-123', status: 'uploaded' }
      }).as('uploadFile')

      cy.get('[data-cy=file-upload]').selectFile('cypress/fixtures/test-orange.jpg')

      // Check progress bar appears
      cy.get('[data-cy=upload-progress]').should('be.visible')

      cy.wait('@uploadFile')

      // Progress should complete
      cy.get('[data-cy=upload-progress]').should('have.attr', 'data-percentage', '100')
    })

    it('should handle upload errors', () => {
      cy.get('[data-cy=nav-upload]').click()

      // Mock upload failure
      cy.intercept('POST', '/api/v1/uploads', {
        statusCode: 500,
        body: { error: 'Upload failed' }
      }).as('uploadFail')

      cy.get('[data-cy=file-upload]').selectFile('cypress/fixtures/test-grape.jpg')

      cy.wait('@uploadFail')

      // Check error message
      cy.get('[data-cy=upload-error]').should('be.visible')
      cy.get('[data-cy=upload-error]').should('contain', '上传失败')
    })
  })

  describe('Live Camera', () => {
    it('should display camera stream', () => {
      cy.get('[data-cy=nav-camera]').click()

      // Mock camera permission
      cy.window().then((win) => {
        cy.stub(win.navigator.mediaDevices, 'getUserMedia').resolves({
          getTracks: () => [{ stop: cy.stub() }]
        })
      })

      cy.get('[data-cy=camera-start]').click()
      cy.get('[data-cy=camera-stream]').should('be.visible')
    })

    it('should capture screenshots', () => {
      cy.get('[data-cy=nav-camera]').click()

      // Start camera (mocked)
      cy.get('[data-cy=camera-start]').click()

      // Take screenshot
      cy.get('[data-cy=camera-capture]').click()

      // Verify screenshot saved
      cy.get('[data-cy=screenshot-gallery]').should('contain', '截图')
    })
  })

  describe('Data Dashboard', () => {
    beforeEach(() => {
      // Mock statistics API
      cy.intercept('GET', '/api/v1/statistics*', {
        fixture: 'statistics-data.json'
      }).as('getStats')
    })

    it('should display statistics cards', () => {
      cy.get('[data-cy=nav-dashboard]').click()

      cy.wait('@getStats')

      // Check all stat cards are present
      cy.get('[data-cy=stat-total-processed]').should('be.visible')
      cy.get('[data-cy=stat-accuracy]').should('be.visible')
      cy.get('[data-cy=stat-defect-rate]').should('be.visible')
      cy.get('[data-cy=stat-avg-time]').should('be.visible')
    })

    it('should display charts', () => {
      cy.get('[data-cy=nav-dashboard]').click()

      cy.wait('@getStats')

      // Check charts are rendered
      cy.get('[data-cy=trend-chart]').should('be.visible')
      cy.get('[data-cy=accuracy-chart]').should('be.visible')
      cy.get('[data-cy=distribution-chart]').should('be.visible')
    })

    it('should allow time range selection', () => {
      cy.get('[data-cy=nav-dashboard]').click()

      cy.wait('@getStats')

      // Change time range
      cy.get('[data-cy=time-range-selector]').select('7d')

      // Verify new request with different range
      cy.wait('@getStats').then((interception) => {
        expect(interception.request.query.range).to.eq('7d')
      })
    })
  })

  describe('Results View', () => {
    beforeEach(() => {
      // Mock results API
      cy.intercept('GET', '/api/v1/results*', {
        fixture: 'results-data.json'
      }).as('getResults')
    })

    it('should display results table', () => {
      cy.get('[data-cy=nav-results]').click()

      cy.wait('@getResults')

      // Check table is populated
      cy.get('[data-cy=results-table]').should('be.visible')
      cy.get('[data-cy=result-row]').should('have.length.greaterThan', 0)
    })

    it('should filter results', () => {
      cy.get('[data-cy=nav-results]').click()

      cy.wait('@getResults')

      // Apply status filter
      cy.get('[data-cy=filter-status]').select('completed')
      cy.get('[data-cy=filter-apply]').click()

      // Verify filtered request
      cy.wait('@getResults').then((interception) => {
        expect(interception.request.query.status).to.eq('completed')
      })
    })

    it('should paginate results', () => {
      cy.get('[data-cy=nav-results]').click()

      cy.wait('@getResults')

      // Go to next page
      cy.get('[data-cy=pagination-next]').click()

      // Verify page change
      cy.wait('@getResults').then((interception) => {
        expect(interception.request.query.page).to.eq('2')
      })
    })

    it('should export results', () => {
      cy.get('[data-cy=nav-results]').click()

      cy.wait('@getResults')

      // Mock export API
      cy.intercept('GET', '/api/v1/results/export*', {
        headers: { 'content-disposition': 'attachment; filename=results.csv' },
        body: 'id,status,created_at\\ntest-1,completed,2023-12-01'
      }).as('exportResults')

      cy.get('[data-cy=export-button]').click()

      cy.wait('@exportResults')

      // Verify download initiated (browser dependent)
      // In a real scenario, you'd check the downloads folder
    })
  })

  describe('Sort Control', () => {
    beforeEach(() => {
      // Mock sort API
      cy.intercept('POST', '/api/v1/sort/command', {
        body: { success: true, message: '操作成功' }
      }).as('sortCommand')

      cy.intercept('GET', '/api/v1/sort/status', {
        fixture: 'sort-status.json'
      }).as('getSortStatus')
    })

    it('should start sorting process', () => {
      cy.get('[data-cy=nav-sort]').click()

      cy.wait('@getSortStatus')

      // Start sorting
      cy.get('[data-cy=sort-start]').click()

      cy.wait('@sortCommand').then((interception) => {
        expect(interception.request.body.action).to.eq('start')
      })

      // Verify success message
      cy.get('[data-cy=sort-message]').should('contain', '操作成功')
    })

    it('should stop sorting process', () => {
      cy.get('[data-cy=nav-sort]').click()

      cy.wait('@getSortStatus')

      // Stop sorting
      cy.get('[data-cy=sort-stop]').click()

      // Confirm stop action
      cy.get('[data-cy=confirm-stop]').click()

      cy.wait('@sortCommand').then((interception) => {
        expect(interception.request.body.action).to.eq('stop')
      })
    })

    it('should update sorting parameters', () => {
      cy.get('[data-cy=nav-sort]').click()

      cy.wait('@getSortStatus')

      // Update conveyor speed
      cy.get('[data-cy=speed-slider]').invoke('val', 75).trigger('input')

      // Update confidence threshold
      cy.get('[data-cy=confidence-input]').clear().type('0.8')

      // Apply changes
      cy.get('[data-cy=apply-params]').click()

      cy.wait('@sortCommand').then((interception) => {
        expect(interception.request.body.conveyor_speed).to.eq(75)
      })
    })
  })

  describe('WebSocket Integration', () => {
    it('should connect to WebSocket and receive updates', () => {
      // Mock WebSocket
      cy.window().then((win) => {
        const mockWs = {
          send: cy.stub(),
          close: cy.stub(),
          onmessage: null,
          onopen: null,
          onerror: null,
          onclose: null
        }

        cy.stub(win, 'WebSocket').returns(mockWs)

        // Visit dashboard to trigger WebSocket connection
        cy.get('[data-cy=nav-dashboard]').click()

        // Simulate WebSocket connection open
        setTimeout(() => {
          mockWs.onopen && mockWs.onopen()
        }, 100)

        // Simulate receiving detection result
        setTimeout(() => {
          mockWs.onmessage && mockWs.onmessage({
            data: JSON.stringify({
              type: 'detection_result',
              data: {
                job_id: 'test-123',
                status: 'completed',
                results: [
                  { class_name: '苹果', confidence: 0.95, quality_grade: 'A' }
                ]
              }
            })
          })
        }, 200)

        // Verify live update indicator
        cy.get('[data-cy=live-status]').should('contain', '已连接')

        // Verify real-time data update
        cy.get('[data-cy=live-results]').should('contain', '苹果')
      })
    })
  })

  describe('System Integration', () => {
    it('should handle complete upload to result flow', () => {
      // Mock APIs
      cy.intercept('POST', '/api/v1/uploads', {
        body: { upload_id: 'integration-test-123', status: 'uploaded' }
      }).as('upload')

      cy.intercept('GET', '/api/v1/results/integration-test-123', {
        body: {
          job_id: 'integration-test-123',
          status: 'completed',
          results: [
            {
              class_name: '苹果',
              confidence: 0.95,
              bbox: [10, 20, 100, 150],
              quality_grade: 'A'
            }
          ],
          created_at: '2023-12-01T10:30:00Z'
        }
      }).as('getResult')

      // Upload file
      cy.get('[data-cy=nav-upload]').click()
      cy.get('[data-cy=file-upload]').selectFile('cypress/fixtures/test-apple.jpg')

      cy.wait('@upload')

      // Navigate to results
      cy.get('[data-cy=view-result]').click()

      cy.wait('@getResult')

      // Verify result details
      cy.get('[data-cy=result-details]').should('be.visible')
      cy.get('[data-cy=detected-class]').should('contain', '苹果')
      cy.get('[data-cy=confidence-score]').should('contain', '95%')
      cy.get('[data-cy=quality-grade]').should('contain', 'A')
    })

    it('should handle error scenarios gracefully', () => {
      // Test network errors
      cy.intercept('POST', '/api/v1/uploads', { forceNetworkError: true }).as('networkError')

      cy.get('[data-cy=nav-upload]').click()
      cy.get('[data-cy=file-upload]').selectFile('cypress/fixtures/test-apple.jpg')

      cy.wait('@networkError')

      // Verify error handling
      cy.get('[data-cy=network-error]').should('be.visible')
      cy.get('[data-cy=retry-button]').should('be.visible')

      // Test retry functionality
      cy.intercept('POST', '/api/v1/uploads', {
        body: { upload_id: 'retry-123', status: 'uploaded' }
      }).as('retryUpload')

      cy.get('[data-cy=retry-button]').click()

      cy.wait('@retryUpload')

      cy.get('[data-cy=upload-success]').should('be.visible')
    })
  })

  describe('Performance Tests', () => {
    it('should handle multiple concurrent uploads', () => {
      cy.get('[data-cy=nav-upload]').click()

      // Mock multiple uploads
      cy.intercept('POST', '/api/v1/uploads', (req) => {
        req.reply({
          delay: Math.random() * 1000, // Random delay 0-1s
          body: {
            upload_id: `concurrent-${Date.now()}-${Math.random()}`,
            status: 'uploaded'
          }
        })
      }).as('concurrentUpload')

      // Upload multiple files
      const files = [
        'cypress/fixtures/test-apple.jpg',
        'cypress/fixtures/test-orange.jpg',
        'cypress/fixtures/test-grape.jpg'
      ]

      files.forEach((file, index) => {
        cy.get('[data-cy=file-upload]').selectFile(file, { force: true })
        cy.wait(500) // Small delay between uploads
      })

      // Verify all uploads complete
      cy.get('[data-cy=upload-success]').should('have.length', files.length)
    })

    it('should maintain responsiveness with large datasets', () => {
      // Mock large dataset
      const largeDataset = Array.from({ length: 1000 }, (_, i) => ({
        job_id: `large-dataset-${i}`,
        status: 'completed',
        total_fruits: Math.floor(Math.random() * 50) + 10,
        created_at: new Date(Date.now() - i * 60000).toISOString()
      }))

      cy.intercept('GET', '/api/v1/results*', {
        body: {
          data: largeDataset.slice(0, 50), // First page
          pagination: {
            page: 1,
            limit: 50,
            total: 1000,
            pages: 20
          }
        }
      }).as('getLargeResults')

      cy.get('[data-cy=nav-results]').click()

      cy.wait('@getLargeResults')

      // Verify table renders without performance issues
      cy.get('[data-cy=results-table]').should('be.visible')
      cy.get('[data-cy=result-row]').should('have.length', 50)

      // Test pagination performance
      const startTime = Date.now()
      cy.get('[data-cy=pagination-next]').click()

      cy.wait('@getLargeResults').then(() => {
        const endTime = Date.now()
        expect(endTime - startTime).to.be.lessThan(2000) // Should be faster than 2s
      })
    })
  })

  describe('Accessibility Tests', () => {
    it('should be keyboard navigable', () => {
      // Test keyboard navigation through main menu
      cy.get('body').tab()
      cy.focused().should('have.attr', 'data-cy', 'nav-upload')

      cy.focused().tab()
      cy.focused().should('have.attr', 'data-cy', 'nav-camera')

      cy.focused().tab()
      cy.focused().should('have.attr', 'data-cy', 'nav-dashboard')

      // Test Enter key activation
      cy.focused().type('{enter}')
      cy.url().should('include', '/dashboard')
    })

    it('should have proper ARIA labels', () => {
      cy.get('[data-cy=nav-upload]').should('have.attr', 'aria-label', '图像上传')
      cy.get('[data-cy=file-upload]').should('have.attr', 'aria-label', '选择文件上传')

      cy.get('[data-cy=nav-dashboard]').click()
      cy.get('[data-cy=stat-total-processed]').should('have.attr', 'aria-label')
    })

    it('should work with screen readers', () => {
      // Test that important elements have proper text content
      cy.get('[data-cy=nav-upload]').should('contain.text', '图像上传')
      cy.get('[data-cy=upload-instruction]').should('contain.text', '将文件拖到此处')

      // Test that status messages are announced
      cy.get('[data-cy=nav-upload]').click()
      cy.get('[data-cy=file-upload]').selectFile('cypress/fixtures/test-apple.jpg')

      cy.get('[data-cy=upload-status]').should('have.attr', 'aria-live', 'polite')
    })
  })
})
