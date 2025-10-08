#!/usr/bin/python3

import sys
import xml.etree.ElementTree as ET


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
    return "%-26s %-10s %10s,%-10s"%(time, typ, lon, lat)



def parseXML(xmlfile):

    # create element tree object
    tree = ET.parse(xmlfile)

    # get root element
    root = tree.getroot()

    x=root.findall('./{http://www.topografix.com/GPX/1/1}trk/{http://www.topografix.com/GPX/1/1}trkseg/')
    print("%s %s %5d %s"%(dump_trppt(x[0]), dump_trppt(x[-1]), len(x), xmlfile))


if __name__ == "__main__":

    # calling main function
    for gpxname in sys.argv[1:]:
        parseXML(gpxname)
