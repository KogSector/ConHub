'use client'

import { useState, useEffect } from 'react'
import { useAuth } from '@/contexts/auth-context'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Download, FileText, Calendar } from 'lucide-react'

interface Invoice {
  id: string
  invoice_number: string
  status: string
  amount_due: number
  amount_paid: number
  currency: string
  due_date?: string
  paid_at?: string
  hosted_invoice_url?: string
  invoice_pdf_url?: string
  created_at: string
}

export function InvoiceHistory() {
  const { token } = useAuth()
  const [invoices, setInvoices] = useState<Invoice[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetchInvoices()
  }, [])

  const fetchInvoices = async () => {
    try {
      const response = await fetch('/api/billing/invoices?limit=20', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      })
      if (response.ok) {
        const invoicesData = await response.json()
        setInvoices(invoicesData)
      }
    } catch (error) {
      console.error('Failed to fetch invoices:', error)
    } finally {
      setLoading(false)
    }
  }

  const getStatusBadge = (status: string) => {
    const variants: Record<string, 'default' | 'secondary' | 'destructive'> = {
      paid: 'default',
      open: 'secondary',
      draft: 'secondary',
      void: 'destructive',
      uncollectible: 'destructive',
    }

    return (
      <Badge variant={variants[status] || 'secondary'}>
        {status.charAt(0).toUpperCase() + status.slice(1)}
      </Badge>
    )
  }

  const formatCurrency = (amount: number, currency: string = 'USD') => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: currency,
    }).format(amount)
  }

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    })
  }

  const handleDownloadInvoice = (invoice: Invoice) => {
    if (invoice.invoice_pdf_url) {
      window.open(invoice.invoice_pdf_url, '_blank')
    } else if (invoice.hosted_invoice_url) {
      window.open(invoice.hosted_invoice_url, '_blank')
    }
  }

  if (loading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Invoice History</CardTitle>
          <CardDescription>Loading your billing history...</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {[...Array(5)].map((_, i) => (
              <div key={i} className="flex items-center justify-between p-4 border rounded animate-pulse">
                <div className="space-y-2">
                  <div className="h-4 bg-muted rounded w-32"></div>
                  <div className="h-3 bg-muted rounded w-24"></div>
                </div>
                <div className="h-6 bg-muted rounded w-16"></div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Invoice History</h2>
        <p className="text-muted-foreground">
          View and download your billing history
        </p>
      </div>

      {invoices.length === 0 ? (
        <Card>
          <CardContent className="p-12 text-center">
            <FileText className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
            <h3 className="text-lg font-semibold mb-2">No Invoices</h3>
            <p className="text-muted-foreground">
              You don't have any invoices yet. Invoices will appear here once you have an active subscription.
            </p>
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <FileText className="h-5 w-5" />
              Invoices
            </CardTitle>
            <CardDescription>
              {invoices.length} invoice{invoices.length !== 1 ? 's' : ''} found
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="hidden md:block">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Invoice</TableHead>
                    <TableHead>Date</TableHead>
                    <TableHead>Due Date</TableHead>
                    <TableHead>Amount</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead className="text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {invoices.map((invoice) => (
                    <TableRow key={invoice.id}>
                      <TableCell className="font-medium">
                        {invoice.invoice_number}
                      </TableCell>
                      <TableCell>
                        {formatDate(invoice.created_at)}
                      </TableCell>
                      <TableCell>
                        {invoice.due_date ? formatDate(invoice.due_date) : 'N/A'}
                      </TableCell>
                      <TableCell>
                        {formatCurrency(invoice.amount_due, invoice.currency)}
                      </TableCell>
                      <TableCell>
                        {getStatusBadge(invoice.status)}
                      </TableCell>
                      <TableCell className="text-right">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleDownloadInvoice(invoice)}
                          disabled={!invoice.invoice_pdf_url && !invoice.hosted_invoice_url}
                        >
                          <Download className="h-4 w-4 mr-2" />
                          Download
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>

            {/* Mobile view */}
            <div className="md:hidden space-y-4">
              {invoices.map((invoice) => (
                <Card key={invoice.id}>
                  <CardContent className="p-4">
                    <div className="flex items-center justify-between mb-2">
                      <h3 className="font-semibold">{invoice.invoice_number}</h3>
                      {getStatusBadge(invoice.status)}
                    </div>
                    <div className="space-y-1 text-sm text-muted-foreground">
                      <div className="flex items-center gap-2">
                        <Calendar className="h-4 w-4" />
                        Created: {formatDate(invoice.created_at)}
                      </div>
                      {invoice.due_date && (
                        <div className="flex items-center gap-2">
                          <Calendar className="h-4 w-4" />
                          Due: {formatDate(invoice.due_date)}
                        </div>
                      )}
                    </div>
                    <div className="flex items-center justify-between mt-4">
                      <span className="text-lg font-semibold">
                        {formatCurrency(invoice.amount_due, invoice.currency)}
                      </span>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleDownloadInvoice(invoice)}
                        disabled={!invoice.invoice_pdf_url && !invoice.hosted_invoice_url}
                      >
                        <Download className="h-4 w-4 mr-2" />
                        Download
                      </Button>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}