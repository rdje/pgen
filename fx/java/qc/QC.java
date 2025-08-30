package qc;

import java.io.*;
import org.apache.poi.poifs.filesystem.*;
import org.apache.poi.hssf.usermodel.*;

public class QC {
 
 public static void main (String[] argv) {
  
//  POIFSFileSystem fs      = new POIFSFileSystem(new FileInputStream("workbook.xls"));
//  HSSFWorkbook wb         = new HSSFWorkbook(fs);
//  HSSFSheet sheet         = wb.getSheetAt(0);
//  HSSFRow row = sheet.getRow(2);
//  HSSFCell cell = row.getCell((short)3);
//  if (cell == null)
//      cell = row.createCell((short)3);
//  cell.setCellType(HSSFCell.CELL_TYPE_STRING);
//  cell.setCellValue("a test");
//
//  // Write the output to a file
//  FileOutputStream fileOut = new FileOutputStream("workbook.xls");
//  wb.write(fileOut);
//  fileOut.close();
  try {
    POIFSFileSystem fs      = new POIFSFileSystem(new FileInputStream("/home/rdje/mpuss_n3g2_xcel2hm_jan30_rev3.xls"));
    HSSFWorkbook wb         = new HSSFWorkbook(fs);

    //short sheetcnt          = (short) wb.getNumberOfSheets();
    //System.out.println("" + sheetcnt + "sheets in '/home/rdje/mpuss_n3g2_xcel2hm_jan30_rev3.xls'");

  } catch (Throwable f) {
  }

 }
}
