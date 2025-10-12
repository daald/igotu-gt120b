#!/usr/bin/python3

import sys
import xml.etree.ElementTree as ET

summary_files = 0
summary_trkpt = 0

def dump_trppt(trkpt):
    #ET.dump(trkpt)
    time=trkpt.find('{http://www.topografix.com/GPX/1/1}time').text
    typ=trkpt.find('{http://www.topografix.com/GPX/1/1}type')
    if typ is not None:
     typ=typ.text
    else:
     typ=""
    lon=trkpt.attrib['lon']
    lat=trkpt.attrib['lat']
    return "%-26s %-10s %7.3f,%7.3f"%(time, typ, float(lon), float(lat))

def parseXML(xmlfile):
    global summary_trkpt, summary_files


    tree = ET.parse(xmlfile)
    root = tree.getroot()
    x=root.findall('./{http://www.topografix.com/GPX/1/1}trk/{http://www.topografix.com/GPX/1/1}trkseg/')
    summary_files += 1
    summary_trkpt += len(x)
    print("%s  %s  %5d %s"%(dump_trppt(x[0]), dump_trppt(x[-1]), len(x), xmlfile))

if __name__ == "__main__":
    for gpxname in sys.argv[1:]:
        parseXML(gpxname)
    print("%-108s  %5d %d"%('', summary_trkpt, summary_files))
