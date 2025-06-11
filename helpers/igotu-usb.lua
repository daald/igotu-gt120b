-- Simple Wireshark Lua dissector for a custom protocol
local my_proto = Proto("igotu", "i-gotU GT-120")

local data_length = ProtoField.uint16("igotu.data_length", "dataLength", base.DEC)
local data = ProtoField.bytes("igotu.data_length", "data")  --, base.RANGE_STRING)
local checksum = ProtoField.uint8("igotu.checksum", "checksum", base.HEX)
my_proto.fields = { data_length, data, checksum }

function my_proto.dissector(buffer, pinfo, tree)
    local urb_type = buffer(8+0, 1):string()
    local urb_transfer_type = buffer(8+1, 1):uint()
    local urb_endpoint_address = buffer(8+2, 1):uint()
    local subtree
    if not buffer(0x40, 1):uint() == 0x93 then return end
    if urb_type=='S' and urb_transfer_type == 0x03 and urb_endpoint_address == 0x01 then
        --pinfo.cols.info = "CMD abc" .. urb_type .. " " .. urb_transfer_type .. " " .. urb_endpoint_address


        --if buffer(0x40, 3) == {0x93,0x01,0x01} then
        if buffer(0x40, 3):bytes():tohex() == "930101" then
        --if buffer(0x40, 3):bytes() == {0x93,0x01,0x01} then
            pinfo.cols.info = "igotu CMD NmeaSwitchCommand(enable = ".. tostring(buffer(3,1) == 0)..")"
        elseif buffer(0x40,2):bytes():tohex() == "930A" then
            --r = unpack_from('<IBB4s', response)
            pinfo.cols.info = "igotu CMD IdentificationCommand()"
            --print('  serialNumber() -> %d' % (r[0]))
            --print('  firmwareVersion() -> %u.%02u' % (r[1], r[2]))
        elseif buffer(0x40,3):bytes():tohex() == "930B03" and buffer(0x40+4,1):int() == 0x1d then
            --    r = unpack_from('>xH', response)
            pinfo.cols.info = "igotu CMD CountCommand()"
            --    print('  trackPointCount() -> %d' % (r[0]))
        elseif buffer(0x40,7):bytes():tohex() == "9305040003019F" then
            --    r = unpack_from('>BH', response)
            pinfo.cols.info = "igotu CMD ModelCommand()"
            --    print('  modelName() -> 0x%06x' % (r[0] * 0x10000 + r[1]))
        elseif buffer(0x40,3):bytes():tohex() == "930507" and buffer(0x40+5,2):bytes():tohex() == "0403" then
            --    r = unpack_from('>xxxHxxBH', query)
            pinfo.cols.info = "igotu CMD ReadCommand" --(pos = 0x%06x, size = 0x%04x)' % (r[1] * 0x10000 +                        r[2], r[0]))
        elseif buffer(0x40,3):bytes():tohex() == "930607" and buffer(0x40+5,1):int() == 0x04 then
            --    r = unpack_from('>xxxHxBBH', query)
            pinfo.cols.info = "igotu CMD WriteCommand"--(mode = 0x%02x, pos = 0x%06x, size = 0x%04x)' % (r[1],                        r[2] * 0x1000 + r[3], r[0]))                rawdatapackages = (r[0] + 6) / 7
        elseif buffer(0x40,3):bytes():tohex() == "930903" then
            --    r = unpack_from('>xxxBBB', query)
            pinfo.cols.info = "igotu CMD TimeCommand"--(time = time(%02u, %02u, %02u)' % (r[0], r[1],                        r[2]))
        elseif buffer(0x40,3):bytes():tohex() == "930C00" then
            --    r = unpack_from('>xxxB', query)
            pinfo.cols.info = "igotu CMD UnknownPurgeCommand1"--(mode = 0x%02x)' % (r[0]))
        elseif buffer(0x40,3):bytes():tohex() == "930802" then
            pinfo.cols.info = "igotu CMD UnknownPurgeCommand2()"
        elseif buffer(0x40,4):bytes():tohex() == "93060400" and buffer(0x40+5,2):bytes():tohex() == "0106" then
            --    r = unpack_from('>xxxxB', query)
            pinfo.cols.info = "igotu CMD UnknownWriteCommand1"--(mode = 0x%02x)' % (r[0]))
        elseif buffer(0x40,3):bytes():tohex() == "930504" and buffer(0x40+5,2):bytes():tohex() == "0105" then
            --    r = unpack_from('>xxxH', query)
            pinfo.cols.info = "igotu CMD UnknownWriteCommand2"--(size = 0x%04x)' % (r[0]))
        elseif buffer(0x40,3):bytes():tohex() == "930D07" then
            pinfo.cols.info = "igotu CMD UnknownWriteCommand3()"
        elseif buffer(0x40,2):bytes():tohex() == "9309" then
            pinfo.cols.info = "igotu CMD (unverified) UnknownTimeCommand()"  -- takes [s] and [ms] epoch timestamp at the same time
        elseif buffer(0x40,3):bytes():tohex() == "931102" then --unverified
            pinfo.cols.info = "igotu CMD (unverified) device reboot"  -- takes [s] and [ms] epoch timestamp at the same time
        else
            pinfo.cols.info = "igotu CMD   (unknown)"
        end

        -- Create a display tree
        subtree = tree:add(my_proto, buffer(), "i-gotU Command Data")
    elseif urb_type == 'C' and urb_transfer_type == 0x03 and urb_endpoint_address == 0x81 then
        pinfo.cols.info = "ANSW abc" .. urb_type .. " " .. urb_transfer_type .. " " .. urb_endpoint_address
        -- Create a display tree
        subtree = tree:add(my_proto, buffer(), "i-gotU Answer Data")
        local payloadlen = buffer(0x41,2):uint()
        subtree:add(data_length, buffer(0x41,2))
        subtree:add(data, buffer(0x43,payloadlen))
        subtree:add(checksum, buffer(0x43+payloadlen,1))
        subtree:add(buffer(0x41,2), "payload len: " .. buffer(0x41,2):uint())
        subtree:add(buffer(0x43,payloadlen), "payload: " .. buffer(0x43,payloadlen))
        subtree:add(buffer(0x43+payloadlen,1), "checksum: " .. buffer(0x43+payloadlen,1))
    else
        return
    end

    pinfo.cols.protocol = my_proto.name


--    local length = buffer:len()
    
    
    -- Add fields to the tree
    subtree:add(buffer(0,1), "Field 1: " .. buffer(0,1):uint())
    subtree:add(buffer(1,2), "Field 2: " .. buffer(1,2):uint())
end

-- Register the dissector for a specific UDP port
--local udp_port = DissectorTable.get("udp.port")
--udp_port:add(1234, my_proto)
--DissectorTable.get("usb.bulk"):add(0xffff, my_proto)

register_postdissector(my_proto) --registers a Postdissector


--usb.idVendor==0x0df7 or (usb.capdata[0] == 0x93 and ((usb.urb_type == 'S' and usb.transfer_type == 0x03 and usb.endpoint_address == 0x01) or (usb.urb_type == 'C' and usb.transfer_type == 0x03 and usb.endpoint_address == 0x81)))








--local my_field = ProtoField.uint16("myproto.fieldname", "Field Description", base.DEC)
--
--function my_proto.dissector(buffer, pinfo, tree)
--    tree:add(my_field, buffer(0, 2))  -- Accessing 2 bytes of data
--end

--myproto.fieldname == 42


--DissectorTable.get("usb.interrupt"):add(0xffff, usb_mouse_protocol)
