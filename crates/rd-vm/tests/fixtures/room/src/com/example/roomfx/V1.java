package com.example.roomfx;

import android.content.Context;
import androidx.room.ColumnInfo;
import androidx.room.Dao;
import androidx.room.Database;
import androidx.room.Entity;
import androidx.room.Insert;
import androidx.room.PrimaryKey;
import androidx.room.Query;
import androidx.room.Room;
import androidx.room.RoomDatabase;

/** Esquema da versão 1: tabela `item` sem `qty`. */
@Entity(tableName = "item")
class ItemV1 {
    @PrimaryKey
    public int id;
    @ColumnInfo(name = "name")
    public String name;
}

@Dao
interface ItemV1Dao {
    @Insert
    void insert(ItemV1 item);

    @Query("SELECT COUNT(*) FROM item")
    int count();
}

@Database(entities = {ItemV1.class}, version = 1, exportSchema = false)
abstract class AppDbV1 extends RoomDatabase {
    abstract ItemV1Dao items();
}

/** Driver da versão 1: cria o banco com uma linha, como um app já instalado. */
public class V1 {
    public static int seed(Context ctx, String name) {
        AppDbV1 db = Room.databaseBuilder(ctx, AppDbV1.class, name).allowMainThreadQueries().build();
        ItemV1 row = new ItemV1();
        row.id = 1;
        row.name = "alpha";
        db.items().insert(row);
        db.close();
        return 1;
    }

    /** Downgrade 2→1 com fallbackToDestructiveMigrationOnDowngrade: recria a tabela da v1. */
    public static String openOverV2(Context ctx, String name) {
        return openV1Over(ctx, name, true);
    }

    /** Downgrade 2→1 sem fallback: o Room real recusa a abertura. */
    public static String openOverV2WithoutFallback(Context ctx, String name) {
        return openV1Over(ctx, name, false);
    }

    private static String openV1Over(Context ctx, String name, boolean onDowngrade) {
        AppDbV1 db;
        if (onDowngrade) {
            db = Room.databaseBuilder(ctx, AppDbV1.class, name)
                    .fallbackToDestructiveMigrationOnDowngrade()
                    .allowMainThreadQueries()
                    .build();
        } else {
            db = Room.databaseBuilder(ctx, AppDbV1.class, name)
                    .allowMainThreadQueries()
                    .build();
        }
        try {
            return "count=" + db.items().count();
        } catch (RuntimeException e) {
            return "error:" + e.getClass().getName() + ":" + e.getMessage();
        } finally {
            db.close();
        }
    }
}
