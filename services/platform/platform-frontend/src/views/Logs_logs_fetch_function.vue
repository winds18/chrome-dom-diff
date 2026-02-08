const fetchLogs = async () => {
  loading.value = true
  try {
    const params: any = {
      level: queryParams.level || undefined,
      limit: queryParams.limit
    }
    const response = await logsApi.getLogs(params)
    if (response.code === 200 && response.data) {
      logs.value = response.data
    }
  } catch (error) {
    console.error('获取日志失败:', error)
  } finally {
    loading.value = false
  }
}
