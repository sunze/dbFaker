use anyhow::{anyhow, Result};
use bincode::{deserialize, serialize};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sled::{Db, Tree};
use std::path::Path;
use uuid::Uuid;

// -------------------------- 数据结构定义 --------------------------
/// 数据集基础信息（对应前端“新建数据集”页面）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dataset {
    pub id: String,          // 数据集唯一ID（UUID v4）
    pub name: String,        // 数据集名称（唯一）
    pub file_path: Option<String>,  // CSV文件路径（可选，如：./uploads/user.csv）
    pub create_time: DateTime<Utc>, // 创建时间（UTC）
    pub update_time: DateTime<Utc>, // 最后更新时间（UTC）
}

/// 数据集内容项（对应前端“编辑数据集内容项”页面）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatasetItem {
    pub id: String,          // 内容项唯一ID（UUID v4）
    pub dataset_id: String,  // 关联的数据集ID（外键）
    pub item_name: String,   // 内容项名称（如：用户ID、用户名）
    pub item_type: String,   // 内容项类型（int/string/date/float）
}

// -------------------------- 数据库常量定义 --------------------------
const DB_DIR: &str = "./sled_dataset_db";  // Sled数据库存储目录
const DATASET_TREE: &str = "datasets";     // 存储数据集的树（key: dataset_id, value: Dataset序列化后）
const DATASET_ITEM_TREE: &str = "dataset_items";  // 存储内容项的树（key: dataset_id:item_id, value: DatasetItem序列化后）
const DATASET_NAME_INDEX: &str = "dataset_name_index";  // 数据集名称索引树（key: name, value: dataset_id，保证名称唯一）

// -------------------------- 数据库操作封装 --------------------------
/// 数据库实例（单例模式，避免重复打开）
#[derive(Debug, Clone)]
pub struct DatasetDb {
    db: Db,
    datasets: Tree,
    dataset_items: Tree,
    name_index: Tree,
}

impl DatasetDb {
    /// 初始化数据库（创建目录和树，不存在则自动创建）
    pub fn init() -> Result<Self> {
        // 打开/创建Sled数据库（目录不存在则自动创建）
        let db = sled::open(DB_DIR)?;

        // 打开/创建三个树（类似MySQL的表）
        let datasets = db.open_tree(DATASET_TREE)?;
        let dataset_items = db.open_tree(DATASET_ITEM_TREE)?;
        let name_index = db.open_tree(DATASET_NAME_INDEX)?;

        Ok(Self {
            db,
            datasets,
            dataset_items,
            name_index,
        })
    }

    // -------------------------- 数据集操作 --------------------------
    /// 新增数据集（返回数据集ID）
    pub fn add_dataset(&self, name: &str, file_path: Option<&str>) -> Result<String> {
        // 1. 检查名称是否已存在（通过索引树）
        if self.name_index.contains_key(name)? {
            return Err(anyhow!("数据集名称“{}”已存在", name));
        }

        // 2. 生成数据集唯一ID和时间
        let dataset_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // 3. 构建数据集结构体
        let dataset = Dataset {
            id: dataset_id.clone(),
            name: name.to_string(),
            file_path: file_path.map(|p| p.to_string()),
            create_time: now,
            update_time: now,
        };

        // 4. 序列化并存储（Sled支持事务，失败自动回滚）
        let dataset_bytes = serialize(&dataset)?;
        self.datasets.insert(&dataset_id, dataset_bytes)?;  // 存储数据集
        self.name_index.insert(name, &dataset_id)?;         // 存储名称索引（保证唯一）
        self.db.flush()?;  // 强制刷盘（可选，Sled默认异步刷盘，重要数据建议手动刷盘）

        Ok(dataset_id)
    }

    /// 根据ID查询数据集
    pub fn get_dataset_by_id(&self, dataset_id: &str) -> Result<Option<Dataset>> {
        // 从数据集树中获取字节数据，反序列化为Dataset
        match self.datasets.get(dataset_id)? {
            Some(bytes) => Ok(Some(deserialize(&bytes)?)),
            None => Ok(None),
        }
    }

    /// 查询所有数据集（分页可选，这里返回全部）
    pub fn get_all_datasets(&self) -> Result<Vec<Dataset>> {
        let mut datasets = Vec::new();
        // 迭代数据集树的所有条目，反序列化为Dataset
        for result in self.datasets.iter() {
            let (_, bytes) = result?;
            datasets.push(deserialize(&bytes)?);
        }
        Ok(datasets)
    }

    /// 删除数据集（关联的内容项也会被删除）
    pub fn delete_dataset(&self, dataset_id: &str) -> Result<()> {
        // 1. 查询数据集是否存在
        let dataset = match self.get_dataset_by_id(dataset_id)? {
            Some(d) => d,
            None => return Err(anyhow!("数据集不存在")),
        };

        // 2. 事务删除：删除数据集 + 名称索引 + 关联的内容项
        let _ = self.datasets.remove(dataset_id)?;  // 删除数据集
        let _ = self.name_index.remove(&dataset.name)?;  // 删除名称索引

        // 3. 删除所有关联的内容项（前缀匹配：dataset_id:）
        let prefix = format!("{}:", dataset_id);
        for result in self.dataset_items.scan_prefix(&prefix) {
            let (key, _) = result?;
            self.dataset_items.remove(key)?;
        }

        self.db.flush()?;
        Ok(())
    }

    // -------------------------- 内容项操作 --------------------------
    /// 新增内容项（单个）
    pub fn add_dataset_item(&self, dataset_id: &str, item_name: &str, item_type: &str) -> Result<String> {
        // 1. 检查数据集是否存在
        if !self.datasets.contains_key(dataset_id)? {
            return Err(anyhow!("数据集不存在"));
        }

        // 2. 生成内容项唯一ID
        let item_id = Uuid::new_v4().to_string();

        // 3. 构建内容项结构体
        let item = DatasetItem {
            id: item_id.clone(),
            dataset_id: dataset_id.to_string(),
            item_name: item_name.to_string(),
            item_type: item_type.to_string(),
        };

        // 4. 存储内容项（key格式：dataset_id:item_id，方便后续按数据集ID查询）
        let key = format!("{}:{}", dataset_id, item_id);
        let item_bytes = serialize(&item)?;
        self.dataset_items.insert(&key, item_bytes)?;
        self.db.flush()?;

        Ok(item_id)
    }

    /// 批量新增内容项（效率更高，适合编辑页面批量保存）
    pub fn batch_add_items(&self, dataset_id: &str, items: &[(String, String)]) -> Result<Vec<String>> {
        // items: (item_name, item_type) 列表
        if !self.datasets.contains_key(dataset_id)? {
            return Err(anyhow!("数据集不存在"));
        }

        let mut item_ids = Vec::with_capacity(items.len());
        for (item_name, item_type) in items {
            let item_id = self.add_dataset_item(dataset_id, item_name, item_type)?;
            item_ids.push(item_id);
        }

        Ok(item_ids)
    }

    /// 根据数据集ID查询所有内容项
    pub fn get_items_by_dataset_id(&self, dataset_id: &str) -> Result<Vec<DatasetItem>> {
        let mut items = Vec::new();
        let prefix = format!("{}:", dataset_id);

        // 前缀匹配查询：所有key以 "dataset_id:" 开头的内容项
        for result in self.dataset_items.scan_prefix(&prefix) {
            let (_, bytes) = result?;
            items.push(deserialize(&bytes)?);
        }

        Ok(items)
    }

    /// 删除单个内容项
    pub fn delete_item(&self, dataset_id: &str, item_id: &str) -> Result<()> {
        let key = format!("{}:{}", dataset_id, item_id);
        if !self.dataset_items.contains_key(&key)? {
            return Err(anyhow!("内容项不存在"));
        }

        self.dataset_items.remove(&key)?;
        self.db.flush()?;
        Ok(())
    }

    /// 批量删除内容项
    pub fn batch_delete_items(&self, dataset_id: &str, item_ids: &[&str]) -> Result<()> {
        for item_id in item_ids {
            self.delete_item(dataset_id, item_id)?;
        }
        Ok(())
    }
}

// -------------------------- 测试代码 --------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_crud() -> Result<()> {
        // 初始化数据库
        let db = DatasetDb::init()?;

        // 1. 新增数据集
        let dataset_id = db.add_dataset("用户信息数据集", Some("./uploads/user.csv"))?;
        assert!(!dataset_id.is_empty());
        println!("新增数据集ID: {}", dataset_id);

        // 2. 查询数据集
        let dataset = db.get_dataset_by_id(&dataset_id)?;
        assert!(dataset.is_some());
        let dataset = dataset.unwrap();
        assert_eq!(dataset.name, "用户信息数据集");
        println!("查询数据集: {:?}", dataset);

        // 3. 新增内容项（批量）
        let items = vec![
            ("用户ID".to_string(), "int".to_string()),
            ("用户名".to_string(), "string".to_string()),
            ("注册时间".to_string(), "date".to_string()),
        ];
        let item_ids = db.batch_add_items(&dataset_id, &items)?;
        assert_eq!(item_ids.len(), 3);
        println!("新增内容项ID: {:?}", item_ids);

        // 4. 查询内容项
        let dataset_items = db.get_items_by_dataset_id(&dataset_id)?;
        assert_eq!(dataset_items.len(), 3);
        println!("查询内容项: {:?}", dataset_items);

        // 5. 删除单个内容项
        db.delete_item(&dataset_id, &item_ids[0])?;
        let dataset_items_after_delete = db.get_items_by_dataset_id(&dataset_id)?;
        assert_eq!(dataset_items_after_delete.len(), 2);
        println!("删除单个内容项后: {:?}", dataset_items_after_delete);

        // 6. 删除数据集（关联内容项也会被删除）
        db.delete_dataset(&dataset_id)?;
        let dataset_after_delete = db.get_dataset_by_id(&dataset_id)?;
        assert!(dataset_after_delete.is_none());
        let items_after_delete = db.get_items_by_dataset_id(&dataset_id)?;
        assert!(items_after_delete.is_empty());
        println!("删除数据集成功");

        Ok(())
    }
}