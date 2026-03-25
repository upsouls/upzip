### UpZip
Библиотека для работы с .zip

Поддерживает все популярные алгоритмы сжатия, включая:
- zstd
- deflate
- lzma
- lzma2
- deflate64
- 
А также шифровку и распаковку зашифрованных архивов.

### Пример использования
```lua
local upzip = require 'upzip'

local passed_tests = 0
local needs_password = 0

local function testUnpackArchiveDeflate()
    upzip.extractZip(love.filesystem.getSaveDirectory()..'/test_1.up', love.filesystem.getSaveDirectory()..'/test_1')
    passed_tests = passed_tests + 1
end

local function testUnpackArchiveDeflatePassword()
    upzip.extractProtectedZip(love.filesystem.getSaveDirectory()..'/test_2.up', love.filesystem.getSaveDirectory()..'/test_2', '8112009')
    passed_tests = passed_tests + 1
end

local function testWriteArchiveDeflate()
    upzip.createZip(love.filesystem.getSaveDirectory()..'/write_test', love.filesystem.getSaveDirectory()..'/write_test.up')
    passed_tests = passed_tests + 1
end

local function testWriteArchivePasswordDeflate()
    upzip.createProtectedZip(love.filesystem.getSaveDirectory()..'/write_test', love.filesystem.getSaveDirectory()..'/write_test_pwd.up', '8112009')
    passed_tests = passed_tests + 1
end

function love.load()
    testUnpackArchiveDeflate()
    testUnpackArchiveDeflatePassword()
    testWriteArchiveDeflate()
    testWriteArchivePasswordDeflate()
    needs_password = upzip.getZipState(love.filesystem.getSaveDirectory()..'/write_test_pwd.up')
end

function love.draw()
    love.graphics.print("Passed tests: "..passed_tests..'\nwrite_test_pwd encrypted: '..tostring(needs_password == upzip.FLAG_NEEDS_PASSWORD)..'('..tostring(needs_password)..')', 10, 10)
end```
