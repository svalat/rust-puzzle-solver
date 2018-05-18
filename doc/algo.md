Algo
====

This page provide a short hint on the algorithm we used to implement the solver.


Step 1 - extract pieces
-----------------------

The first step is to detect the position of each pieces in the picture and compute the square which surround it. We can then extract it into a sub-picture.

In order to proceed we consider there is no overlap between the surrounding square, meaning there is enought space betweeen all the pieces in the picture.

The algorithm work like this :
 * First search the first non white (background color which can be something else).
  * From this we check each next lines and remember the xmin and xmax position with detected pixels from object.
  * For each line we search new interesting pixel on the left of xmin and right of xmax then update xmin/xmax.
  * We then check if the line between xmin/xmax contain at least one interesting pixel. If not we reach the end of the object on the Y axis.

We then color the selecte background in green and the object in blue to help debugging. We also ignore those specific color from the search for next object as if it is background.

Step 2 - Rotate objects
-----------------------

In order to well extract the border pattern for matching with other pieces we need to rotate the piece to put it into an optimal position.

To find the nice rotation angle we proceed by search the rectangle which surround the share with the higher ratio between larger side and smaller side.
This seams to work quite well in practice because most pieces are rectangular.

The algorithm the find the angle run by doing :
 * Calculate the middle point of the image (which is more or less the middle of the piece)
 * We then loop on all angles between 0 and 90 degree
 * For each angle we build the rectangle from the middle by searching the first border line which does not cover any interesting pixel.
 * We do this by testing all the lines looping on an offset to move from the center to the border with the given angle.
 * we do this with the 4 sides of the rectangle turned by the given angle, one by one.
 * We can then extract the ratio of larger side divided by smaller side. The larger ratio for all the tested angle give the best solution.